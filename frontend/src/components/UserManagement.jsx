import React from 'react';
import { useAuth } from '../contexts/AuthContext';
import LoginForm from './LoginForm';
import UserInfo from './UserInfo';

const UserManagement = () => {
  const { user, isLoading } = useAuth();

  if (isLoading) {
    return (
      <div className="auth-loading">
        <div className="loading-spinner"></div>
        <p>Loading...</p>
      </div>
    );
  }

  if (!user) {
    return <LoginForm />;
  }

  return <UserInfo />;
};

export default UserManagement;