import React, { useState, useEffect } from 'react';
import { useAuth } from '../contexts/AuthContext';
import LoginForm from './LoginForm';
import UserInfo from './UserInfo';

const UserManagement = () => {
  const { user, isLoading, sessionToken } = useAuth();
  const [loadingProgress, setLoadingProgress] = useState(0);
  const [loadingMessage, setLoadingMessage] = useState('');

  // Enhanced loading states with progress simulation
  useEffect(() => {
    if (isLoading) {
      let progress = 0;
      const messages = [
        'Initializing authentication...',
        'Validating credentials...',
        'Loading user profile...',
        'Setting up session...',
        'Almost ready...'
      ];
      
      const interval = setInterval(() => {
        progress += Math.random() * 30;
        if (progress > 100) {
          progress = 100;
          clearInterval(interval);
        }
        
        setLoadingProgress(Math.min(progress, 100));
        
        const messageIndex = Math.min(
          Math.floor((progress / 100) * messages.length),
          messages.length - 1
        );
        setLoadingMessage(messages[messageIndex]);
      }, 200);

      return () => clearInterval(interval);
    } else {
      setLoadingProgress(0);
      setLoadingMessage('');
    }
  }, [isLoading]);

  // Enhanced loading screen with progress
  if (isLoading) {
    return (
      <div className="auth-loading-container">
        <div className="auth-loading-content">
          <div className="auth-loading-header">
            <h1 className="system-logo">🚨 Alert System</h1>
            <p className="loading-subtitle">Secure Authentication</p>
          </div>
          
          <div className="loading-progress-container">
            <div className="loading-progress-bar">
              <div 
                className="loading-progress-fill"
                style={{ width: `${loadingProgress}%` }}
              ></div>
            </div>
            <div className="loading-percentage">
              {Math.round(loadingProgress)}%
            </div>
          </div>
          
          <div className="loading-status">
            <div className="enhanced-loading-spinner"></div>
            <p className="loading-message">{loadingMessage}</p>
          </div>
          
          <div className="loading-details">
            <div className="loading-detail-item">
              <span className="detail-icon">🔐</span>
              <span>Session: {sessionToken ? 'Active' : 'Initializing'}</span>
            </div>
            <div className="loading-detail-item">
              <span className="detail-icon">🌐</span>
              <span>Connection: Secure</span>
            </div>
            <div className="loading-detail-item">
              <span className="detail-icon">⚡</span>
              <span>Status: {loadingProgress === 100 ? 'Ready' : 'Loading'}</span>
            </div>
          </div>
        </div>
      </div>
    );
  }

  // Render appropriate component based on authentication state
  if (!user) {
    return <LoginForm />;
  }

  return <UserInfo />;
};

export default UserManagement;