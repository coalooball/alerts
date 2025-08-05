import React, { useState } from 'react';
import { useAuth } from '../contexts/AuthContext';
import ProfileSettingsModal from './ProfileSettingsModal';
import SecuritySettingsModal from './SecuritySettingsModal';

const UserInfo = () => {
  const { user, logout, isLoading } = useAuth();
  const [isLoggingOut, setIsLoggingOut] = useState(false);
  const [showUserMenu, setShowUserMenu] = useState(false);
  const [showProfileModal, setShowProfileModal] = useState(false);
  const [showSecurityModal, setShowSecurityModal] = useState(false);

  if (isLoading) {
    return (
      <div className="user-info loading">
        <div className="loading-spinner"></div>
        <span>Loading user...</span>
      </div>
    );
  }

  if (!user) {
    return null;
  }

  const handleLogout = async () => {
    if (isLoggingOut) return;
    
    const confirmLogout = window.confirm('Are you sure you want to logout?');
    if (!confirmLogout) return;

    setIsLoggingOut(true);
    try {
      await logout();
    } catch (error) {
      console.error('Logout failed:', error);
      alert('Logout failed. Please try again.');
    } finally {
      setIsLoggingOut(false);
    }
  };

  const formatDate = (dateStr) => {
    if (!dateStr) return 'Unknown';
    return new Date(dateStr).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    });
  };

  return (
    <div className="user-info-container">
      <div 
        className="user-profile"
        onClick={() => setShowUserMenu(!showUserMenu)}
        title="Click to view profile"
      >
        <div className="user-avatar">
          <span className="avatar-icon">👤</span>
        </div>
        <div className="user-details">
          <span className="username">{user.username}</span>
          <span className="user-role">{user.role || 'User'}</span>
        </div>
        <div className="dropdown-arrow">
          <span>{showUserMenu ? '▲' : '▼'}</span>
        </div>
      </div>

      {showUserMenu && (
        <div className="user-menu-dropdown">
          <div className="user-menu-header">
            <div className="user-full-info">
              <h4>{user.username}</h4>
              {user.email && <p className="user-email">{user.email}</p>}
              <div className="user-session-info">
                <small>Role: {user.role || 'User'}</small>
                {user.last_login && (
                  <small>Last login: {formatDate(user.last_login)}</small>
                )}
              </div>
            </div>
          </div>
          
          <div className="user-menu-actions">
            <button 
              className="menu-action-btn profile-btn"
              onClick={(e) => {
                e.stopPropagation();
                setShowUserMenu(false);
                setShowProfileModal(true);
              }}
            >
              <span>⚙️</span>
              Profile Settings
            </button>
            
            <button 
              className="menu-action-btn security-btn"
              onClick={(e) => {
                e.stopPropagation();
                setShowUserMenu(false);
                setShowSecurityModal(true);
              }}
            >
              <span>🔒</span>
              Security
            </button>
            
            <hr className="menu-divider" />
            
            <button 
              onClick={handleLogout}
              className="menu-action-btn logout-btn"
              disabled={isLoggingOut}
            >
              <span>{isLoggingOut ? '⏳' : '🚪'}</span>
              {isLoggingOut ? 'Logging out...' : 'Logout'}
            </button>
          </div>
        </div>
      )}

      {/* Profile Settings Modal */}
      <ProfileSettingsModal
        isOpen={showProfileModal}
        onClose={() => setShowProfileModal(false)}
      />

      {/* Security Settings Modal */}
      <SecuritySettingsModal
        isOpen={showSecurityModal}
        onClose={() => setShowSecurityModal(false)}
      />
    </div>
  );
};

export default UserInfo;