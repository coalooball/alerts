import React from 'react';
import { useAuth } from '../contexts/AuthContext';

const UserInfo = () => {
  const { user, logout, isLoading } = useAuth();

  if (isLoading) {
    return <div className="user-info loading">Loading...</div>;
  }

  if (!user) {
    return null;
  }

  const handleLogout = async () => {
    try {
      await logout();
    } catch (error) {
      console.error('Logout failed:', error);
    }
  };

  return (
    <div className="user-info">
      <div className="user-details">
        <span className="username">👤 {user.username}</span>
        <span className="email">{user.email}</span>
      </div>
      <button 
        onClick={handleLogout}
        className="logout-button"
        title="Logout"
      >
        Logout
      </button>
    </div>
  );
};

export default UserInfo;