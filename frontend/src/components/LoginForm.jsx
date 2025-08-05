import React, { useState, useEffect } from 'react';
import { useAuth } from '../contexts/AuthContext';

const LoginForm = () => {
  const [credentials, setCredentials] = useState({
    username: '',
    password: ''
  });
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState('');
  const [showPassword, setShowPassword] = useState(false);
  const [focusedField, setFocusedField] = useState('');
  const [validationErrors, setValidationErrors] = useState({});

  const { login } = useAuth();

  // Clear error when user starts typing
  useEffect(() => {
    if (error && (credentials.username || credentials.password)) {
      setError('');
    }
  }, [credentials.username, credentials.password, error]);

  const validateForm = () => {
    const errors = {};
    
    if (!credentials.username.trim()) {
      errors.username = 'Username is required';
    } else if (credentials.username.length < 3) {
      errors.username = 'Username must be at least 3 characters';
    }
    
    if (!credentials.password) {
      errors.password = 'Password is required';
    } else if (credentials.password.length < 6) {
      errors.password = 'Password must be at least 6 characters';
    }
    
    setValidationErrors(errors);
    return Object.keys(errors).length === 0;
  };

  const handleSubmit = async (e) => {
    e.preventDefault();
    setError('');
    setValidationErrors({});

    if (!validateForm()) {
      return;
    }

    setIsLoading(true);

    try {
      await login(credentials.username.trim(), credentials.password);
    } catch (err) {
      let errorMessage = 'Login failed';
      
      if (err.message) {
        if (err.message.includes('Invalid credentials')) {
          errorMessage = 'Invalid username or password';
        } else if (err.message.includes('Network')) {
          errorMessage = 'Network error. Please check your connection.';
        } else {
          errorMessage = err.message;
        }
      }
      
      setError(errorMessage);
    } finally {
      setIsLoading(false);
    }
  };

  const handleChange = (e) => {
    const { name, value } = e.target;
    setCredentials(prev => ({
      ...prev,
      [name]: value
    }));
    
    // Clear field-specific validation error
    if (validationErrors[name]) {
      setValidationErrors(prev => ({
        ...prev,
        [name]: ''
      }));
    }
  };

  const handleFocus = (fieldName) => {
    setFocusedField(fieldName);
  };

  const handleBlur = () => {
    setFocusedField('');
  };

  const isFormValid = credentials.username.trim() && credentials.password.length >= 6;

  return (
    <div className="login-container">
      <div className="login-form-wrapper">
        <div className="login-header">
          <h1 className="system-title">🚨 Security Alert System</h1>
          <h2 className="login-title">Welcome Back</h2>
          <p className="login-subtitle">Please sign in to access the dashboard</p>
        </div>

        <form onSubmit={handleSubmit} className="enhanced-login-form" noValidate>
          {error && (
            <div className="error-message" role="alert" aria-live="polite">
              <span className="error-icon">⚠️</span>
              {error}
            </div>
          )}

          <div className="form-group">
            <label htmlFor="username" className="form-label">
              Username
              <span className="required-asterisk" aria-label="required">*</span>
            </label>
            <div className={`input-wrapper ${focusedField === 'username' ? 'focused' : ''} ${validationErrors.username ? 'error' : ''}`}>
              <span className="input-icon">👤</span>
              <input
                type="text"
                id="username"
                name="username"
                value={credentials.username}
                onChange={handleChange}
                onFocus={() => handleFocus('username')}
                onBlur={handleBlur}
                required
                disabled={isLoading}
                className="form-input"
                placeholder="Enter your username"
                autoComplete="username"
                aria-describedby={validationErrors.username ? "username-error" : undefined}
                aria-invalid={validationErrors.username ? "true" : "false"}
              />
            </div>
            {validationErrors.username && (
              <div id="username-error" className="field-error" role="alert">
                {validationErrors.username}
              </div>
            )}
          </div>

          <div className="form-group">
            <label htmlFor="password" className="form-label">
              Password
              <span className="required-asterisk" aria-label="required">*</span>
            </label>
            <div className={`input-wrapper ${focusedField === 'password' ? 'focused' : ''} ${validationErrors.password ? 'error' : ''}`}>
              <span className="input-icon">🔒</span>
              <input
                type={showPassword ? "text" : "password"}
                id="password"
                name="password"
                value={credentials.password}
                onChange={handleChange}
                onFocus={() => handleFocus('password')}
                onBlur={handleBlur}
                required
                disabled={isLoading}
                className="form-input"
                placeholder="Enter your password"
                autoComplete="current-password"
                aria-describedby={validationErrors.password ? "password-error" : undefined}
                aria-invalid={validationErrors.password ? "true" : "false"}
              />
              <button
                type="button"
                className="password-toggle"
                onClick={() => setShowPassword(!showPassword)}
                disabled={isLoading}
                aria-label={showPassword ? "Hide password" : "Show password"}
              >
                {showPassword ? '🙈' : '👁️'}
              </button>
            </div>
            {validationErrors.password && (
              <div id="password-error" className="field-error" role="alert">
                {validationErrors.password}
              </div>
            )}
          </div>

          <button 
            type="submit" 
            className={`enhanced-login-button ${!isFormValid ? 'disabled' : ''}`}
            disabled={isLoading || !isFormValid}
            aria-describedby="login-status"
          >
            {isLoading ? (
              <>
                <span className="loading-spinner"></span>
                Signing in...
              </>
            ) : (
              <>
                <span className="login-icon">🚀</span>
                Sign In
              </>
            )}
          </button>

          <div id="login-status" className="sr-only" aria-live="polite">
            {isLoading ? 'Signing in, please wait...' : ''}
          </div>
        </form>

        <div className="login-info">
          <div className="login-help">
            <p>Need help? Contact your system administrator</p>
          </div>
        </div>
      </div>
    </div>
  );
};

export default LoginForm;