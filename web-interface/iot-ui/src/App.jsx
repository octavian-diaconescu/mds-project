import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import Login from './pages/Login';
import Register from './pages/Register';
import Dashboard from './pages/Dashboard';
import Device from './pages/Device';
import Navbar from './nav';

function App() {
  const isAuthenticated = !!localStorage.getItem('token');

  return (
    <BrowserRouter>
     {isAuthenticated && <Navbar />}
     <div className={`${isAuthenticated ? 'pt-16' : ''}`}>

      <Routes>
        <Route path="/login" element={<Login />} />
        <Route path="/register" element={<Register />} />
        <Route path="/dashboard" element={<Dashboard />} />
        <Route path="/device/:thing_name" element={<Device />} />
        <Route path="/" element={<Navigate to="/login" replace />} />
      </Routes>
      </div>
    </BrowserRouter>
  );
}

export default App;