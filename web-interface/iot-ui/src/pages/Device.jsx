import { useParams } from 'react-router-dom';
import { useState, useRef, useEffect } from 'react';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip } from 'recharts';
import api from '../api/client';

export default function Device() {
  const { thing_name } = useParams();
  const [running, setRunning] = useState(false);
  const [data, setData] = useState([]);
  const [error, setError] = useState('');
  const [anomalyCount, setAnomalyCount] = useState(0);
  const [anomalyMessages, setAnomalyMessages] = useState([]);
  const timer = useRef(null);

  const MAX_DATA_POINTS = 20;
  const MAX_ANOMALIES = 5;
  const MAX_MESSAGES = 5;

  const startSim = () => {
    setRunning(true);
    setAnomalyCount(0);
    setAnomalyMessages([]);
    sendDataPoint();
    timer.current = setInterval(sendDataPoint, 5000);
  };

  const stopSim = () => {
    if (timer.current) {
      clearInterval(timer.current);
      timer.current = null;
      setRunning(false);
    }
  };

  const sendDataPoint = () => {
    const temperature = Number((20 + Math.random() * 30).toFixed(2));
    const timestamp = new Date().toLocaleTimeString();

    const payload = {
      device_id: thing_name,
      temperature: temperature,
      timestamp: Math.floor(Date.now() / 1000)
    };

    // Update chart data
    setData(prevData => {
      const newData = [...prevData, { timestamp, temperature }];
      return newData.slice(-MAX_DATA_POINTS);
    });

    // Check for anomaly
    if (temperature > 40) {
      setAnomalyCount(prevCount => {
        const newCount = prevCount + 1;

        // Create message first to avoid duplication
        const message = newCount > MAX_ANOMALIES - 1
          ? 'Too many anomalies found. IoT device will now shutdown. Please check for intrusions.'
          : `Temperature anomaly detected: ${temperature}°C (Anomaly ${newCount}/${MAX_ANOMALIES})`;

        // Update messages only once
        setAnomalyMessages(prev => {
          // Check if this exact message already exists
          const isDuplicate = prev.some(item => item.message === message);
          if (isDuplicate) return prev;
          return [...prev, { timestamp, message }].slice(-MAX_MESSAGES);
        });

        // Handle max anomalies case
        if (newCount > MAX_ANOMALIES  - 1) {
          stopSim();
          setError(message);
        } else {
          setError(message);
        }

        return newCount;
      });
    } else {
      setError('');
    }

    // Send data to backend
    api.post('/api/data', payload)
      .then(response => {
        console.log('Data sent successfully:', response.data);
      })
      .catch(err => {
        console.error('Failed to send data:', err.response?.data || err.message);
        if (err.response?.status === 400) {
          console.error('Invalid payload:', payload);
        } else {
          stopSim();
          setError(`Connection error: ${err.message}`);
        }
      });
  };

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (timer.current) {
        clearInterval(timer.current);
      }
    };
  }, []);

  return (
    <div className="container mx-auto p-6">
      <div className="bg-gray-800 rounded-lg shadow-lg p-6">
        <div className="flex justify-between items-center mb-6">
          <h1 className="text-2xl font-bold text-white">Device: {thing_name}</h1>
          {!running ? (
            <button
              onClick={startSim}
              className="px-6 py-2 bg-green-600 hover:bg-green-700 text-white rounded-lg transition-colors"
            >
              Start Simulation
            </button>
          ) : (
            <button
              onClick={stopSim}
              className="px-6 py-2 bg-red-600 hover:bg-red-700 text-white rounded-lg transition-colors"
            >
              Stop Simulation
            </button>
          )}
        </div>

        {error && (
          <div className="mb-6 p-4 bg-red-500 text-white rounded-lg">
            {error}
          </div>
        )}

        <div className="bg-gray-900 p-4 rounded-lg">
          <LineChart width={800} height={400} data={data}>
            <CartesianGrid strokeDasharray="3 3" />
            <XAxis
              dataKey="timestamp"
              stroke="#fff"
              tick={{ fill: '#fff' }}
            />
            <YAxis
              stroke="#fff"
              tick={{ fill: '#fff' }}
              domain={[0, 60]}
              label={{
                value: 'Temperature (°C)',
                angle: -90,
                position: 'insideLeft',
                fill: '#fff'
              }}
            />
            <Tooltip
              contentStyle={{ backgroundColor: '#1f2937' }}
              labelStyle={{ color: '#fff' }}
            />
            <Line
              type="monotone"
              dataKey="temperature"
              stroke="#3b82f6"
              strokeWidth={2}
              dot={false}
            />
          </LineChart>
        </div>

        {anomalyMessages.length > 0 && (
          <div className="mt-6">
            <h2 className="text-xl font-bold text-white mb-4">Anomaly History</h2>
            <div className="space-y-2">
              {anomalyMessages.map((item, index) => (
                <div
                  key={index}
                  className="bg-gray-700 p-3 rounded-lg text-white flex justify-between items-center"
                >
                  <span>{item.message}</span>
                  <span className="text-gray-400 text-sm">{item.timestamp}</span>
                </div>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}