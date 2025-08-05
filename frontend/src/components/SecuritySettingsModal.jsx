import React, { useState, useEffect } from 'react';
import { useAuth } from '../contexts/AuthContext';

const SecuritySettingsModal = ({ isOpen, onClose }) => {
  const { user, sessionToken } = useAuth();
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState('');
  const [success, setSuccess] = useState('');
  const [formErrors, setFormErrors] = useState({});
  
  const [passwordForm, setPasswordForm] = useState({
    currentPassword: '',
    newPassword: '',
    confirmPassword: ''
  });

  const [securitySettings, setSecuritySettings] = useState({
    twoFactorEnabled: false,
    sessionTimeout: 24,  // hours
    allowMultipleSessions: true,
    requirePasswordChange: false,
    loginNotifications: true
  });

  const [sessions, setSessions] = useState([]);
  const [loadingSessions, setLoadingSessions] = useState(false);

  useEffect(() => {
    if (isOpen) {
      loadSecuritySettings();
      loadActiveSessions();
    }
  }, [isOpen]);

  const loadSecuritySettings = async () => {
    try {
      const response = await fetch('/api/security-settings', {
        headers: {
          'Authorization': `Bearer ${sessionToken}`,
        },
      });

      const data = await response.json();
      if (data.success) {
        setSecuritySettings(prev => ({
          ...prev,
          ...data.settings
        }));
      }
    } catch (err) {
      console.error('Error loading security settings:', err);
    }
  };

  const loadActiveSessions = async () => {
    setLoadingSessions(true);
    try {
      const response = await fetch('/api/sessions', {
        headers: {
          'Authorization': `Bearer ${sessionToken}`,
        },
      });

      const data = await response.json();
      if (data.success) {
        setSessions(data.sessions || []);
      }
    } catch (err) {
      console.error('Error loading sessions:', err);
    } finally {
      setLoadingSessions(false);
    }
  };

  const validatePasswordForm = () => {
    const errors = {};
    
    if (!passwordForm.currentPassword) {
      errors.currentPassword = 'Current password is required';
    }
    
    if (!passwordForm.newPassword) {
      errors.newPassword = 'New password is required';
    } else if (passwordForm.newPassword.length < 6) {
      errors.newPassword = 'New password must be at least 6 characters';
    }
    
    if (!passwordForm.confirmPassword) {
      errors.confirmPassword = 'Please confirm your new password';
    } else if (passwordForm.newPassword !== passwordForm.confirmPassword) {
      errors.confirmPassword = 'Passwords do not match';
    }
    
    setFormErrors(errors);
    return Object.keys(errors).length === 0;
  };

  const getPasswordStrength = (password) => {
    if (!password) return { level: 0, text: '', color: '#e2e8f0' };
    
    let score = 0;
    const checks = {
      length: password.length >= 8,
      lowercase: /[a-z]/.test(password),
      uppercase: /[A-Z]/.test(password),
      numbers: /\d/.test(password),
      special: /[!@#$%^&*(),.?":{}|<>]/.test(password)
    };
    
    score = Object.values(checks).filter(Boolean).length;
    
    if (password.length < 6) {
      return { level: 0, text: 'Too short (minimum 6 characters)', color: '#e53e3e' };
    } else if (score <= 2) {
      return { level: 1, text: 'Weak', color: '#e53e3e' };
    } else if (score <= 3) {
      return { level: 2, text: 'Fair', color: '#d69e2e' };
    } else if (score <= 4) {
      return { level: 3, text: 'Good', color: '#38a169' };
    } else {
      return { level: 4, text: 'Strong', color: '#38a169' };
    }
  };

  const handlePasswordChange = async (e) => {
    e.preventDefault();
    
    if (!validatePasswordForm()) {
      return;
    }

    setIsLoading(true);
    setError('');
    setSuccess('');

    try {
      const response = await fetch('/api/change-password', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${sessionToken}`,
        },
        body: JSON.stringify({
          currentPassword: passwordForm.currentPassword,
          newPassword: passwordForm.newPassword
        }),
      });

      const data = await response.json();

      if (data.success) {
        setSuccess('Password changed successfully!');
        setPasswordForm({
          currentPassword: '',
          newPassword: '',
          confirmPassword: ''
        });
        setFormErrors({});
      } else {
        setError(data.message || 'Failed to change password');
      }
    } catch (err) {
      setError('Failed to change password. Please try again.');
      console.error('Error changing password:', err);
    } finally {
      setIsLoading(false);
    }
  };

  const handleSecuritySettingsChange = async (setting, value) => {
    const newSettings = { ...securitySettings, [setting]: value };
    setSecuritySettings(newSettings);

    try {
      const response = await fetch('/api/security-settings', {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${sessionToken}`,
        },
        body: JSON.stringify({ [setting]: value }),
      });

      const data = await response.json();
      if (!data.success) {
        // Revert on failure
        setSecuritySettings(securitySettings);
        setError(data.message || 'Failed to update security settings');
      }
    } catch (err) {
      // Revert on failure
      setSecuritySettings(securitySettings);
      setError('Failed to update security settings');
      console.error('Error updating security settings:', err);
    }
  };

  const terminateSession = async (sessionId) => {
    try {
      const response = await fetch(`/api/sessions/${sessionId}`, {
        method: 'DELETE',
        headers: {
          'Authorization': `Bearer ${sessionToken}`,
        },
      });

      const data = await response.json();
      if (data.success) {
        setSessions(sessions.filter(s => s.id !== sessionId));
      } else {
        setError(data.message || 'Failed to terminate session');
      }
    } catch (err) {
      setError('Failed to terminate session');
      console.error('Error terminating session:', err);
    }
  };

  const terminateAllOtherSessions = async () => {
    if (!confirm('Are you sure you want to terminate all other sessions? This will log out all other devices.')) {
      return;
    }

    try {
      const response = await fetch('/api/sessions/terminate-others', {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${sessionToken}`,
        },
      });

      const data = await response.json();
      if (data.success) {
        loadActiveSessions(); // Reload sessions
        setSuccess('All other sessions have been terminated');
      } else {
        setError(data.message || 'Failed to terminate sessions');
      }
    } catch (err) {
      setError('Failed to terminate sessions');
      console.error('Error terminating sessions:', err);
    }
  };

  const handlePasswordInputChange = (field, value) => {
    setPasswordForm(prev => ({ ...prev, [field]: value }));
    
    // Clear field-specific error when user starts typing
    if (formErrors[field]) {
      setFormErrors(prev => ({ ...prev, [field]: '' }));
    }
    
    // Clear general messages
    if (error) setError('');
    if (success) setSuccess('');
  };

  const formatLastActive = (timestamp) => {
    if (!timestamp) return 'Unknown';
    const date = new Date(timestamp);
    const now = new Date();
    const diffMs = now - date;
    const diffMins = Math.floor(diffMs / 60000);
    
    if (diffMins < 1) return 'Just now';
    if (diffMins < 60) return `${diffMins} minutes ago`;
    
    const diffHours = Math.floor(diffMins / 60);
    if (diffHours < 24) return `${diffHours} hours ago`;
    
    const diffDays = Math.floor(diffHours / 24);
    return `${diffDays} days ago`;
  };

  const handleClose = () => {
    setError('');
    setSuccess('');
    setFormErrors({});
    onClose();
  };

  if (!isOpen) return null;

  return (
    <div className="modal-overlay">
      <div className="modal-content security-settings-modal">
        <div className="modal-header">
          <h3>Security Settings</h3>
          <button className="modal-close" onClick={handleClose}>
            ×
          </button>
        </div>

        <div className="modal-body">
          {error && (
            <div className="error-message" role="alert">
              <span className="error-icon">⚠️</span>
              {error}
            </div>
          )}

          {success && (
            <div className="success-message" role="alert">
              <span className="success-icon">✅</span>
              {success}
            </div>
          )}

          {/* Password Change Section */}
          <div className="security-section">
            <h4 className="section-title">Change Password</h4>
            <form onSubmit={handlePasswordChange} className="password-form">
              <div className="form-group">
                <label htmlFor="currentPassword">Current Password</label>
                <input
                  type="password"
                  id="currentPassword"
                  value={passwordForm.currentPassword}
                  onChange={(e) => handlePasswordInputChange('currentPassword', e.target.value)}
                  className={formErrors.currentPassword ? 'error' : ''}
                  placeholder="Enter current password"
                />
                {formErrors.currentPassword && (
                  <div className="field-error">{formErrors.currentPassword}</div>
                )}
              </div>

              <div className="form-group">
                <label htmlFor="newPassword">New Password</label>
                <input
                  type="password"
                  id="newPassword"
                  value={passwordForm.newPassword}
                  onChange={(e) => handlePasswordInputChange('newPassword', e.target.value)}
                  className={formErrors.newPassword ? 'error' : ''}
                  placeholder="Enter new password"
                />
                {formErrors.newPassword && (
                  <div className="field-error">{formErrors.newPassword}</div>
                )}
                
                {/* Password Strength Indicator */}
                {passwordForm.newPassword && (
                  <div className="password-strength">
                    <div className="password-strength-bar">
                      <div 
                        className="password-strength-fill"
                        style={{ 
                          width: `${(getPasswordStrength(passwordForm.newPassword).level / 4) * 100}%`,
                          backgroundColor: getPasswordStrength(passwordForm.newPassword).color
                        }}
                      ></div>
                    </div>
                    <div 
                      className="password-strength-text"
                      style={{ color: getPasswordStrength(passwordForm.newPassword).color }}
                    >
                      {getPasswordStrength(passwordForm.newPassword).text}
                    </div>
                  </div>
                )}
              </div>

              <div className="form-group">
                <label htmlFor="confirmPassword">Confirm New Password</label>
                <input
                  type="password"
                  id="confirmPassword"
                  value={passwordForm.confirmPassword}
                  onChange={(e) => handlePasswordInputChange('confirmPassword', e.target.value)}
                  className={formErrors.confirmPassword ? 'error' : ''}
                  placeholder="Confirm new password"
                />
                {formErrors.confirmPassword && (
                  <div className="field-error">{formErrors.confirmPassword}</div>
                )}
              </div>

              <button
                type="submit"
                className="btn btn-primary"
                disabled={isLoading || Object.keys(formErrors).length > 0}
              >
                {isLoading ? (
                  <>
                    <span className="loading-spinner"></span>
                    Changing Password...
                  </>
                ) : (
                  <>
                    <span className="lock-icon">🔒</span>
                    Change Password
                  </>
                )}
              </button>
            </form>
          </div>

          {/* Security Settings Section */}
          <div className="security-section">
            <h4 className="section-title">Security Preferences</h4>
            
            <div className="security-options">
              <div className="security-option">
                <label className="security-label">
                  <input
                    type="checkbox"
                    checked={securitySettings.twoFactorEnabled}
                    onChange={(e) => handleSecuritySettingsChange('twoFactorEnabled', e.target.checked)}
                  />
                  <span className="security-text">Two-Factor Authentication</span>
                  <span className="security-description">Add an extra layer of security to your account</span>
                </label>
              </div>

              <div className="security-option">
                <label className="security-label">
                  <input
                    type="checkbox"
                    checked={securitySettings.allowMultipleSessions}
                    onChange={(e) => handleSecuritySettingsChange('allowMultipleSessions', e.target.checked)}
                  />
                  <span className="security-text">Allow Multiple Sessions</span>
                  <span className="security-description">Allow login from multiple devices simultaneously</span>
                </label>
              </div>

              <div className="security-option">
                <label className="security-label">
                  <input
                    type="checkbox"
                    checked={securitySettings.loginNotifications}
                    onChange={(e) => handleSecuritySettingsChange('loginNotifications', e.target.checked)}
                  />
                  <span className="security-text">Login Notifications</span>
                  <span className="security-description">Get notified when your account is accessed</span>
                </label>
              </div>

              <div className="security-option">
                <label className="security-label-inline">
                  <span className="security-text">Session Timeout</span>
                  <select
                    value={securitySettings.sessionTimeout}
                    onChange={(e) => handleSecuritySettingsChange('sessionTimeout', parseInt(e.target.value))}
                    className="security-select"
                  >
                    <option value={1}>1 hour</option>
                    <option value={4}>4 hours</option>
                    <option value={8}>8 hours</option>
                    <option value={24}>24 hours</option>
                    <option value={168}>1 week</option>
                  </select>
                </label>
                <span className="security-description">Automatically log out after this period of inactivity</span>
              </div>
            </div>
          </div>

          {/* Active Sessions Section */}
          <div className="security-section">
            <div className="section-header">
              <h4 className="section-title">Active Sessions</h4>
              <button
                className="btn btn-warning btn-sm"
                onClick={terminateAllOtherSessions}
                disabled={loadingSessions}
              >
                Terminate All Others
              </button>
            </div>

            {loadingSessions ? (
              <div className="loading-sessions">
                <div className="loading-spinner"></div>
                <span>Loading sessions...</span>
              </div>
            ) : (
              <div className="sessions-list">
                {sessions.length === 0 ? (
                  <div className="no-sessions">No active sessions found</div>
                ) : (
                  sessions.map((session) => (
                    <div key={session.id} className={`session-item ${session.current ? 'current-session' : ''}`}>
                      <div className="session-info">
                        <div className="session-device">
                          <span className="device-icon">
                            {session.device_type === 'mobile' ? '📱' : 
                             session.device_type === 'tablet' ? '📱' : '💻'}
                          </span>
                          <div className="device-details">
                            <div className="device-name">
                              {session.device_name || 'Unknown Device'}
                              {session.current && <span className="current-badge">Current</span>}
                            </div>
                            <div className="device-info">
                              {session.ip_address} • {session.location || 'Unknown Location'}
                            </div>
                          </div>
                        </div>
                        <div className="session-time">
                          <div className="last-active">
                            Last active: {formatLastActive(session.last_active)}
                          </div>
                          <div className="session-created">
                            Created: {new Date(session.created_at).toLocaleDateString()}
                          </div>
                        </div>
                      </div>
                      {!session.current && (
                        <button
                          className="btn btn-danger btn-sm"
                          onClick={() => terminateSession(session.id)}
                        >
                          Terminate
                        </button>
                      )}
                    </div>
                  ))
                )}
              </div>
            )}
          </div>
        </div>

        <div className="modal-footer">
          <button
            type="button"
            className="btn btn-secondary"
            onClick={handleClose}
          >
            Close
          </button>
        </div>
      </div>
    </div>
  );
};

export default SecuritySettingsModal;