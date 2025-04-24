import { useEffect, useState } from 'react';
import api from '../api/client';
import { useNavigate } from 'react-router-dom';

export default function Dashboard() {
  const [devices, setDevices] = useState([]);
  const [error, setError] = useState('');
  const [isCreating, setIsCreating] = useState(false);
  const [newDeviceName, setNewDeviceName] = useState('');
  const navigate = useNavigate();

  const fetchDevices = async () => {
    try {
      const userId = localStorage.getItem('userId');
      const response = await api.get('/api/devices');
      console.log('Devices response:', response.data);
      setDevices(response.data);
    } catch (err) {
      console.error('Error fetching devices:', err);
      if (err.response?.status === 401) {
        localStorage.removeItem('token');
        localStorage.removeItem('userId');
        navigate('/login');
      } else {
        setError('Failed to fetch devices: ' + (err.response?.data || err.message));
      }
    }
  };

  useEffect(() => {
    const token = localStorage.getItem('token');
    const userId = localStorage.getItem('userId');
    if (!token || !userId) {
      navigate('/login');
      return;
    }

    api.defaults.headers.common['Authorization'] = `Bearer ${token}`;
    fetchDevices();
  }, [navigate]);

  const createDevice = async (e) => {
    e.preventDefault();
    setError('');
    setIsCreating(true);

    try {
      const userId = localStorage.getItem('userId');
      const response = await api.post('/api/devices', { name: newDeviceName });
      setDevices([...devices, response.data]);
      setNewDeviceName('');
      setIsCreating(false);
    } catch (err) {
      console.error('Error creating device:', err);
      setError(err.response?.data || 'Failed to create device');
    }
  };

  const deleteDevice = async (thingName, e) => {
    e.stopPropagation(); // Prevent navigation when clicking delete
    if (!confirm(`Are you sure you want to delete ${thingName}?`)) return;

    try {
      const userId = localStorage.getItem('userId');
      await api.delete(`/api/devices/${thingName}`);
      setDevices(devices.filter(device => device.thing_name !== thingName));
    } catch (err) {
      setError(err.response?.data || 'Failed to delete device');
    }
  };

  return (
    <div className="min-h-screen bg-gray-900 text-white p-8">
      <div className="max-w-6xl mx-auto">
        <div className="flex justify-between items-center mb-6">
          <h1 className="text-3xl font-bold">Your Devices</h1>
          <button
            onClick={() => setIsCreating(!isCreating)}
            className="bg-blue-600 hover:bg-blue-700 px-4 py-2 rounded-lg transition"
          >
            {isCreating ? 'Cancel' : 'Add Device'}
          </button>
        </div>

        {error && (
          <div className="bg-red-500 text-white p-3 rounded-lg mb-4">
            {error}
          </div>
        )}

        {isCreating && (
          <form onSubmit={createDevice} className="bg-gray-800 p-4 rounded-lg mb-6">
            <div className="flex gap-4">
              <input
                type="text"
                value={newDeviceName}
                onChange={(e) => setNewDeviceName(e.target.value)}
                placeholder="Enter device name"
                className="flex-1 bg-gray-700 text-white px-4 py-2 rounded border border-gray-600 focus:border-blue-500 focus:outline-none"
                required
              />
              <button
                type="submit"
                className="bg-green-600 hover:bg-green-700 px-4 py-2 rounded transition"
              >
                Create Device
              </button>
            </div>
          </form>
        )}

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {devices.map(device => (
            <div 
              key={device.thing_name} 
              className="bg-gray-800 p-4 rounded-lg cursor-pointer hover:bg-gray-700 transition-colors"
              onClick={() => navigate(`/device/${device.thing_name}`)}
            >
              <div className="flex justify-between items-start">
                <div>
                  <h2 className="text-xl font-semibold">{device.thing_name}</h2>
                  <p className="text-gray-400 text-sm mt-1">Click to view details</p>
                </div>
                <button
                  onClick={(e) => deleteDevice(device.thing_name, e)}
                  className="text-red-500 hover:text-red-400 transition"
                >
                  Delete
                </button>
              </div>
            </div>
          ))}
        </div>

        {devices.length === 0 && !error && (
          <div className="text-center text-gray-400 mt-8">
            No devices found. Create your first device to get started.
          </div>
        )}
      </div>
    </div>
  );
}