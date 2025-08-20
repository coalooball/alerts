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
      errors.username = '用户名不能为空';
    } else if (credentials.username.length < 3) {
      errors.username = '用户名至少需要3个字符';
    }
    
    if (!credentials.password) {
      errors.password = '密码不能为空';
    } else if (credentials.password.length < 6) {
      errors.password = '密码至少需要6个字符';
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
      let errorMessage = '登录失败';
      
      if (err.message) {
        if (err.message.includes('Invalid credentials')) {
          errorMessage = '用户名或密码错误';
        } else if (err.message.includes('Network')) {
          errorMessage = '网络错误，请检查您的网络连接';
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
          <h1 className="system-title">🚨 安全告警系统</h1>
          <h2 className="login-title">欢迎回来</h2>
          <p className="login-subtitle">请登录以访问控制台</p>
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
              用户名
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
                placeholder="请输入用户名"
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
              密码
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
                placeholder="请输入密码"
                autoComplete="current-password"
                aria-describedby={validationErrors.password ? "password-error" : undefined}
                aria-invalid={validationErrors.password ? "true" : "false"}
              />
              <button
                type="button"
                className="password-toggle"
                onClick={() => setShowPassword(!showPassword)}
                disabled={isLoading}
                aria-label={showPassword ? "隐藏密码" : "显示密码"}
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
                登录中...
              </>
            ) : (
              <>
                <span className="login-icon">🚀</span>
                登录
              </>
            )}
          </button>

          <div id="login-status" className="sr-only" aria-live="polite">
            {isLoading ? '正在登录，请稍候...' : ''}
          </div>
        </form>

        <div className="login-info">
          <div className="login-help">
            <p>需要帮助？请联系系统管理员</p>
          </div>
        </div>
      </div>
    </div>
  );
};

export default LoginForm;