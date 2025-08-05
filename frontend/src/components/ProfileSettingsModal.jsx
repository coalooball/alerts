import React, { useState, useEffect } from 'react';
import { useAuth } from '../contexts/AuthContext';
import { useToast } from '../contexts/ToastContext';

const ProfileSettingsModal = ({ isOpen, onClose }) => {
  const { user, sessionToken, refreshUser } = useAuth();
  const { showSuccess, showError } = useToast();
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState('');
  const [success, setSuccess] = useState('');
  const [formErrors, setFormErrors] = useState({});
  
  const [profileForm, setProfileForm] = useState({
    username: '',
    email: '',
    fullName: '',
    phone: '',
    department: '',
    timezone: 'UTC',
    language: 'en',
    emailNotifications: true,
    smsNotifications: false
  });

  useEffect(() => {
    if (isOpen && user) {
      setProfileForm({
        username: user.username || '',
        email: user.email || '',
        fullName: user.full_name || '',
        phone: user.phone || '',
        department: user.department || '',
        timezone: user.timezone || 'UTC',
        language: user.language || 'en',
        emailNotifications: user.email_notifications !== false,
        smsNotifications: user.sms_notifications === true
      });
    }
  }, [isOpen, user]);

  const validateForm = () => {
    const errors = {};
    
    if (!profileForm.username.trim()) {
      errors.username = 'Username is required';
    } else if (profileForm.username.length < 3) {
      errors.username = 'Username must be at least 3 characters';
    }
    
    if (!profileForm.email.trim()) {
      errors.email = 'Email is required';
    } else if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(profileForm.email)) {
      errors.email = 'Please enter a valid email address';
    }

    if (profileForm.phone && !/^[\+]?[\d\s\-\(\)]{10,}$/.test(profileForm.phone)) {
      errors.phone = 'Please enter a valid phone number';
    }
    
    setFormErrors(errors);
    return Object.keys(errors).length === 0;
  };

  const handleInputChange = (field, value) => {
    setProfileForm(prev => ({ ...prev, [field]: value }));
    
    // Clear field-specific error when user starts typing
    if (formErrors[field]) {
      setFormErrors(prev => ({ ...prev, [field]: '' }));
    }
    
    // Clear general messages
    if (error) setError('');
    if (success) setSuccess('');
  };

  const handleSubmit = async (e) => {
    e.preventDefault();
    
    if (!validateForm()) {
      return;
    }

    setIsLoading(true);
    setError('');
    setSuccess('');

    try {
      const response = await fetch('/api/profile', {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${sessionToken}`,
        },
        body: JSON.stringify({
          username: profileForm.username,
          email: profileForm.email,
          fullName: profileForm.fullName,
          phone: profileForm.phone,
          department: profileForm.department,
          timezone: profileForm.timezone,
          language: profileForm.language,
          emailNotifications: profileForm.emailNotifications,
          smsNotifications: profileForm.smsNotifications
        }),
      });

      const data = await response.json();

      if (data.success) {
        showSuccess('Profile updated successfully!');
        // Refresh user data in context
        await refreshUser();
        // Close modal immediately
        handleClose();
      } else {
        setError(data.message || 'Failed to update profile');
      }
    } catch (err) {
      showError('Failed to update profile. Please try again.');
      console.error('Error updating profile:', err);
    } finally {
      setIsLoading(false);
    }
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
      <div className="modal-content profile-settings-modal">
        <div className="modal-header">
          <h3>Profile Settings</h3>
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

          <form onSubmit={handleSubmit} className="profile-form">
            <div className="form-section">
              <h4 className="section-title">Basic Information</h4>
              
              <div className="form-row">
                <div className="form-group">
                  <label htmlFor="username">Username *</label>
                  <input
                    type="text"
                    id="username"
                    value={profileForm.username}
                    onChange={(e) => handleInputChange('username', e.target.value)}
                    className={formErrors.username ? 'error' : ''}
                    placeholder="Enter username"
                  />
                  {formErrors.username && (
                    <div className="field-error">{formErrors.username}</div>
                  )}
                </div>

                <div className="form-group">
                  <label htmlFor="email">Email Address *</label>
                  <input
                    type="email"
                    id="email"
                    value={profileForm.email}
                    onChange={(e) => handleInputChange('email', e.target.value)}
                    className={formErrors.email ? 'error' : ''}
                    placeholder="Enter email address"
                  />
                  {formErrors.email && (
                    <div className="field-error">{formErrors.email}</div>
                  )}
                </div>
              </div>

              <div className="form-row">
                <div className="form-group">
                  <label htmlFor="fullName">Full Name</label>
                  <input
                    type="text"
                    id="fullName"
                    value={profileForm.fullName}
                    onChange={(e) => handleInputChange('fullName', e.target.value)}
                    placeholder="Enter full name"
                  />
                </div>

                <div className="form-group">
                  <label htmlFor="phone">Phone Number</label>
                  <input
                    type="tel"
                    id="phone"
                    value={profileForm.phone}
                    onChange={(e) => handleInputChange('phone', e.target.value)}
                    className={formErrors.phone ? 'error' : ''}
                    placeholder="Enter phone number"
                  />
                  {formErrors.phone && (
                    <div className="field-error">{formErrors.phone}</div>
                  )}
                </div>
              </div>

              <div className="form-group">
                <label htmlFor="department">Department</label>
                <input
                  type="text"
                  id="department"
                  value={profileForm.department}
                  onChange={(e) => handleInputChange('department', e.target.value)}
                  placeholder="Enter department"
                />
              </div>
            </div>

            <div className="form-section">
              <h4 className="section-title">Preferences</h4>
              
              <div className="form-row">
                <div className="form-group">
                  <label htmlFor="timezone">Timezone</label>
                  <select
                    id="timezone"
                    value={profileForm.timezone}
                    onChange={(e) => handleInputChange('timezone', e.target.value)}
                  >
                    <option value="UTC">UTC</option>
                    <option value="Asia/Shanghai">Asia/Shanghai</option>
                    <option value="America/New_York">America/New_York</option>
                    <option value="Europe/London">Europe/London</option>
                    <option value="Asia/Tokyo">Asia/Tokyo</option>
                  </select>
                </div>

                <div className="form-group">
                  <label htmlFor="language">Language</label>
                  <select
                    id="language"
                    value={profileForm.language}
                    onChange={(e) => handleInputChange('language', e.target.value)}
                  >
                    <option value="en">English</option>
                    <option value="zh">中文</option>
                    <option value="ja">日本語</option>
                  </select>
                </div>
              </div>
            </div>

            <div className="form-section">
              <h4 className="section-title">Notifications</h4>
              
              <div className="checkbox-group">
                <label className="checkbox-label">
                  <input
                    type="checkbox"
                    checked={profileForm.emailNotifications}
                    onChange={(e) => handleInputChange('emailNotifications', e.target.checked)}
                  />
                  <span className="checkbox-text">Email Notifications</span>
                  <span className="checkbox-description">Receive alerts and updates via email</span>
                </label>
              </div>

              <div className="checkbox-group">
                <label className="checkbox-label">
                  <input
                    type="checkbox"
                    checked={profileForm.smsNotifications}
                    onChange={(e) => handleInputChange('smsNotifications', e.target.checked)}
                  />
                  <span className="checkbox-text">SMS Notifications</span>
                  <span className="checkbox-description">Receive critical alerts via SMS</span>
                </label>
              </div>
            </div>
          </form>
        </div>

        <div className="modal-footer">
          <button
            type="button"
            className="btn btn-secondary"
            onClick={handleClose}
          >
            Cancel
          </button>
          <button
            type="submit"
            className="btn btn-primary"
            onClick={handleSubmit}
            disabled={isLoading || Object.keys(formErrors).length > 0}
          >
            {isLoading ? (
              <>
                <span className="loading-spinner"></span>
                Updating...
              </>
            ) : (
              <>
                <span className="save-icon">💾</span>
                Save Changes
              </>
            )}
          </button>
        </div>
      </div>
    </div>
  );
};

export default ProfileSettingsModal;