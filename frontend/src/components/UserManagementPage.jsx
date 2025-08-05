import React, { useState, useEffect } from 'react';
import { useAuth } from '../contexts/AuthContext';

const UserManagementPage = () => {
  const { user, sessionToken } = useAuth();
  const [users, setUsers] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [editingUser, setEditingUser] = useState(null);
  const [showPasswordForm, setShowPasswordForm] = useState(null);

  // Form validation states
  const [createFormErrors, setCreateFormErrors] = useState({});
  const [passwordFormErrors, setPasswordFormErrors] = useState({});

  const [createForm, setCreateForm] = useState({
    username: '',
    email: '',
    password: '',
    role: 'user'
  });

  const [editForm, setEditForm] = useState({
    username: '',
    email: '',
    role: 'user',
    is_active: true
  });

  const [passwordForm, setPasswordForm] = useState({
    password: ''
  });

  useEffect(() => {
    loadUsers();
  }, []);

  // Validation functions
  const validateCreateForm = () => {
    const errors = {};
    
    if (!createForm.username.trim()) {
      errors.username = 'Username is required';
    } else if (createForm.username.length < 3) {
      errors.username = 'Username must be at least 3 characters';
    }
    
    if (!createForm.email.trim()) {
      errors.email = 'Email is required';
    } else if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(createForm.email)) {
      errors.email = 'Please enter a valid email address';
    }
    
    if (!createForm.password) {
      errors.password = 'Password is required';
    } else if (createForm.password.length < 6) {
      errors.password = 'Password must be at least 6 characters';
    }
    
    setCreateFormErrors(errors);
    return Object.keys(errors).length === 0;
  };

  const validatePasswordForm = () => {
    const errors = {};
    
    if (!passwordForm.password) {
      errors.password = 'Password is required';
    } else if (passwordForm.password.length < 6) {
      errors.password = 'Password must be at least 6 characters';
    }
    
    setPasswordFormErrors(errors);
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

  // Handle input changes with real-time validation
  const handleCreateFormChange = (field, value) => {
    setCreateForm(prev => ({ ...prev, [field]: value }));
    
    // Clear field-specific error when user starts typing
    if (createFormErrors[field]) {
      setCreateFormErrors(prev => ({
        ...prev,
        [field]: ''
      }));
    }
  };

  const handlePasswordFormChange = (field, value) => {
    setPasswordForm(prev => ({ ...prev, [field]: value }));
    
    // Clear field-specific error when user starts typing
    if (passwordFormErrors[field]) {
      setPasswordFormErrors(prev => ({
        ...prev,
        [field]: ''
      }));
    }
  };

  const apiCall = async (url, options = {}) => {
    const response = await fetch(url, {
      ...options,
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${sessionToken}`,
        ...options.headers,
      },
    });
    return response.json();
  };

  const loadUsers = async () => {
    try {
      setLoading(true);
      const data = await apiCall('/api/users');
      if (data.success) {
        setUsers(data.users);
      } else {
        setError(data.message || 'Failed to load users');
      }
    } catch (err) {
      setError('Failed to load users');
      console.error('Error loading users:', err);
    } finally {
      setLoading(false);
    }
  };

  const handleCreateUser = async (e) => {
    e.preventDefault();
    
    if (!validateCreateForm()) {
      return;
    }

    try {
      const data = await apiCall('/api/users', {
        method: 'POST',
        body: JSON.stringify(createForm),
      });

      if (data.success) {
        setCreateForm({ username: '', email: '', password: '', role: 'user' });
        setCreateFormErrors({});
        setShowCreateForm(false);
        loadUsers();
      } else {
        setError(data.message || 'Failed to create user');
      }
    } catch (err) {
      setError('Failed to create user');
      console.error('Error creating user:', err);
    }
  };

  const handleUpdateUser = async (e) => {
    e.preventDefault();
    try {
      const data = await apiCall(`/api/users/${editingUser.id}`, {
        method: 'PUT',
        body: JSON.stringify(editForm),
      });

      if (data.success) {
        setEditingUser(null);
        setEditForm({ username: '', email: '', role: 'user', is_active: true });
        loadUsers();
      } else {
        setError(data.message || 'Failed to update user');
      }
    } catch (err) {
      setError('Failed to update user');
      console.error('Error updating user:', err);
    }
  };

  const handleUpdatePassword = async (e) => {
    e.preventDefault();
    
    if (!validatePasswordForm()) {
      return;
    }

    try {
      const data = await apiCall(`/api/users/${showPasswordForm.id}/password`, {
        method: 'PUT',
        body: JSON.stringify(passwordForm),
      });

      if (data.success) {
        setShowPasswordForm(null);
        setPasswordForm({ password: '' });
        setPasswordFormErrors({});
      } else {
        setError(data.message || 'Failed to update password');
      }
    } catch (err) {
      setError('Failed to update password');
      console.error('Error updating password:', err);
    }
  };

  const handleDeleteUser = async (userId) => {
    if (!confirm('Are you sure you want to delete this user?')) return;

    try {
      const data = await apiCall(`/api/users/${userId}`, {
        method: 'DELETE',
      });

      if (data.success) {
        loadUsers();
      } else {
        setError(data.message || 'Failed to delete user');
      }
    } catch (err) {
      setError('Failed to delete user');
      console.error('Error deleting user:', err);
    }
  };

  const startEdit = (userToEdit) => {
    setEditingUser(userToEdit);
    setEditForm({
      username: userToEdit.username,
      email: userToEdit.email,
      role: userToEdit.role,
      is_active: userToEdit.is_active
    });
  };

  const startPasswordChange = (userToEdit) => {
    setShowPasswordForm(userToEdit);
    setPasswordForm({ password: '' });
  };

  if (loading) {
    return (
      <div className="user-management-loading">
        <div className="loading-spinner"></div>
        <p>Loading users...</p>
      </div>
    );
  }

  return (
    <div className="user-management">
      <div className="user-management-header">
        <h2>User Management</h2>
        <button 
          className="create-user-button"
          onClick={() => setShowCreateForm(true)}
        >
          + Create User
        </button>
      </div>

      {error && (
        <div className="error-message">
          {error}
          <button onClick={() => setError('')}>×</button>
        </div>
      )}

      {/* Create User Form */}
      {showCreateForm && (
        <div className="modal-overlay">
          <div className="modal">
            <h3>Create New User</h3>
            <form onSubmit={handleCreateUser}>
              <div className="form-group">
                <label>Username:</label>
                <input
                  type="text"
                  value={createForm.username}
                  onChange={(e) => handleCreateFormChange('username', e.target.value)}
                  className={createFormErrors.username ? 'error' : ''}
                  placeholder="Enter username (min 3 characters)"
                />
                {createFormErrors.username && (
                  <div className="field-error">{createFormErrors.username}</div>
                )}
              </div>
              
              <div className="form-group">
                <label>Email:</label>
                <input
                  type="email"
                  value={createForm.email}
                  onChange={(e) => handleCreateFormChange('email', e.target.value)}
                  className={createFormErrors.email ? 'error' : ''}
                  placeholder="Enter email address"
                />
                {createFormErrors.email && (
                  <div className="field-error">{createFormErrors.email}</div>
                )}
              </div>
              
              <div className="form-group">
                <label>Password:</label>
                <input
                  type="password"
                  value={createForm.password}
                  onChange={(e) => handleCreateFormChange('password', e.target.value)}
                  className={createFormErrors.password ? 'error' : ''}
                  placeholder="Enter password (min 6 characters)"
                />
                {createFormErrors.password && (
                  <div className="field-error">{createFormErrors.password}</div>
                )}
                
                {/* Password Strength Indicator */}
                {createForm.password && (
                  <div className="password-strength">
                    <div className="password-strength-bar">
                      <div 
                        className="password-strength-fill"
                        style={{ 
                          width: `${(getPasswordStrength(createForm.password).level / 4) * 100}%`,
                          backgroundColor: getPasswordStrength(createForm.password).color
                        }}
                      ></div>
                    </div>
                    <div 
                      className="password-strength-text"
                      style={{ color: getPasswordStrength(createForm.password).color }}
                    >
                      {getPasswordStrength(createForm.password).text}
                    </div>
                  </div>
                )}
                
                {/* Password Requirements */}
                <div className="password-requirements">
                  <div className="requirement-title">Password Requirements:</div>
                  <div className="requirements-list">
                    <div className={`requirement ${createForm.password.length >= 6 ? 'met' : ''}`}>
                      ✓ At least 6 characters
                    </div>
                    <div className={`requirement ${/[a-z]/.test(createForm.password) ? 'met' : ''}`}>
                      ✓ Lowercase letter (recommended)
                    </div>
                    <div className={`requirement ${/[A-Z]/.test(createForm.password) ? 'met' : ''}`}>
                      ✓ Uppercase letter (recommended)
                    </div>
                    <div className={`requirement ${/\d/.test(createForm.password) ? 'met' : ''}`}>
                      ✓ Number (recommended)
                    </div>
                  </div>
                </div>
              </div>
              
              <div className="form-group">
                <label>Role:</label>
                <select
                  value={createForm.role}
                  onChange={(e) => handleCreateFormChange('role', e.target.value)}
                >
                  <option value="user">User</option>
                  <option value="admin">Admin</option>
                </select>
              </div>
              
              <div className="form-actions">
                <button 
                  type="submit"
                  disabled={Object.keys(createFormErrors).length > 0 || !createForm.username || !createForm.email || !createForm.password}
                >
                  Create User
                </button>
                <button type="button" onClick={() => {
                  setShowCreateForm(false);
                  setCreateForm({ username: '', email: '', password: '', role: 'user' });
                  setCreateFormErrors({});
                }}>Cancel</button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* Edit User Form */}
      {editingUser && (
        <div className="modal-overlay">
          <div className="modal">
            <h3>Edit User</h3>
            <form onSubmit={handleUpdateUser}>
              <div className="form-group">
                <label>Username:</label>
                <input
                  type="text"
                  value={editForm.username}
                  onChange={(e) => setEditForm({...editForm, username: e.target.value})}
                  required
                />
              </div>
              <div className="form-group">
                <label>Email:</label>
                <input
                  type="email"
                  value={editForm.email}
                  onChange={(e) => setEditForm({...editForm, email: e.target.value})}
                  required
                />
              </div>
              <div className="form-group">
                <label>Role:</label>
                <select
                  value={editForm.role}
                  onChange={(e) => setEditForm({...editForm, role: e.target.value})}
                >
                  <option value="user">User</option>
                  <option value="admin">Admin</option>
                </select>
              </div>
              <div className="form-group">
                <label>
                  <input
                    type="checkbox"
                    checked={editForm.is_active}
                    onChange={(e) => setEditForm({...editForm, is_active: e.target.checked})}
                  />
                  Active
                </label>
              </div>
              <div className="form-actions">
                <button type="submit">Update User</button>
                <button type="button" onClick={() => setEditingUser(null)}>Cancel</button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* Change Password Form */}
      {showPasswordForm && (
        <div className="modal-overlay">
          <div className="modal">
            <h3>Change Password for {showPasswordForm.username}</h3>
            <form onSubmit={handleUpdatePassword}>
              <div className="form-group">
                <label>New Password:</label>
                <input
                  type="password"
                  value={passwordForm.password}
                  onChange={(e) => handlePasswordFormChange('password', e.target.value)}
                  className={passwordFormErrors.password ? 'error' : ''}
                  placeholder="Enter new password (min 6 characters)"
                />
                {passwordFormErrors.password && (
                  <div className="field-error">{passwordFormErrors.password}</div>
                )}
                
                {/* Password Strength Indicator */}
                {passwordForm.password && (
                  <div className="password-strength">
                    <div className="password-strength-bar">
                      <div 
                        className="password-strength-fill"
                        style={{ 
                          width: `${(getPasswordStrength(passwordForm.password).level / 4) * 100}%`,
                          backgroundColor: getPasswordStrength(passwordForm.password).color
                        }}
                      ></div>
                    </div>
                    <div 
                      className="password-strength-text"
                      style={{ color: getPasswordStrength(passwordForm.password).color }}
                    >
                      {getPasswordStrength(passwordForm.password).text}
                    </div>
                  </div>
                )}
                
                {/* Password Requirements */}
                <div className="password-requirements">
                  <div className="requirement-title">Password Requirements:</div>
                  <div className="requirements-list">
                    <div className={`requirement ${passwordForm.password.length >= 6 ? 'met' : ''}`}>
                      ✓ At least 6 characters
                    </div>
                    <div className={`requirement ${/[a-z]/.test(passwordForm.password) ? 'met' : ''}`}>
                      ✓ Lowercase letter (recommended)
                    </div>
                    <div className={`requirement ${/[A-Z]/.test(passwordForm.password) ? 'met' : ''}`}>
                      ✓ Uppercase letter (recommended)
                    </div>
                    <div className={`requirement ${/\d/.test(passwordForm.password) ? 'met' : ''}`}>
                      ✓ Number (recommended)
                    </div>
                  </div>
                </div>
              </div>
              
              <div className="form-actions">
                <button 
                  type="submit"
                  disabled={Object.keys(passwordFormErrors).length > 0 || !passwordForm.password}
                >
                  Update Password
                </button>
                <button type="button" onClick={() => {
                  setShowPasswordForm(null);
                  setPasswordForm({ password: '' });
                  setPasswordFormErrors({});
                }}>Cancel</button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* Users Table */}
      <div className="users-table-container">
        <table className="users-table">
          <thead>
            <tr>
              <th>Username</th>
              <th>Email</th>
              <th>Role</th>
              <th>Status</th>
              <th>Created</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {users.map(userItem => (
              <tr key={userItem.id}>
                <td>{userItem.username}</td>
                <td>{userItem.email}</td>
                <td>
                  <span className={`role-badge ${userItem.role}`}>
                    {userItem.role}
                  </span>
                </td>
                <td>
                  <span className={`status-badge ${userItem.is_active ? 'active' : 'inactive'}`}>
                    {userItem.is_active ? 'Active' : 'Inactive'}
                  </span>
                </td>
                <td>{new Date(userItem.created_at).toLocaleDateString()}</td>
                <td>
                  <div className="user-actions">
                    <button 
                      className="edit-button"
                      onClick={() => startEdit(userItem)}
                    >
                      Edit
                    </button>
                    <button 
                      className="password-button"
                      onClick={() => startPasswordChange(userItem)}
                    >
                      Password
                    </button>
                    {userItem.id !== user.id && (
                      <button 
                        className="delete-button"
                        onClick={() => handleDeleteUser(userItem.id)}
                      >
                        Delete
                      </button>
                    )}
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
};

export default UserManagementPage;