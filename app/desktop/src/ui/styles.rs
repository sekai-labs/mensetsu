pub const APP_CSS: &str = r#"
:root {
  --bg-color: #0f172a;
  --surface-color: #1e293b;
  --surface-border: #334155;
  --text-main: #f8fafc;
  --text-muted: #94a3b8;
  --primary: #3b82f6;
  --primary-hover: #2563eb;
  --destructive: #ef4444;
  --destructive-hover: #dc2626;
  --success: #10b981;
  --border-radius: 8px;
  --font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
}

* {
  box-sizing: border-box;
  margin: 0;
  padding: 0;
}

body {
  background-color: var(--bg-color);
  color: var(--text-main);
  font-family: var(--font-family);
  font-size: 14px;
  line-height: 1.5;
  user-select: none;
  overflow-x: hidden;
}

input, textarea, button, select {
  font-family: inherit;
  font-size: inherit;
  border-radius: var(--border-radius);
}

input, textarea, select {
  background-color: var(--surface-color);
  color: var(--text-main);
  border: 1px solid var(--surface-border);
  padding: 8px 12px;
  outline: none;
  width: 100%;
}

input:focus, textarea:focus, select:focus {
  border-color: var(--primary);
  box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.3);
}

button {
  cursor: pointer;
  padding: 8px 16px;
  border: 1px solid transparent;
  font-weight: 500;
  transition: background-color 0.15s, border-color 0.15s;
}

button.primary {
  background-color: var(--primary);
  color: #ffffff;
}

button.primary:hover {
  background-color: var(--primary-hover);
}

button.secondary {
  background-color: transparent;
  border-color: var(--surface-border);
  color: var(--text-main);
}

button.secondary:hover {
  background-color: var(--surface-border);
}

button.destructive {
  background-color: var(--destructive);
  color: #ffffff;
}

button.destructive:hover {
  background-color: var(--destructive-hover);
}

.center-container {
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  min-height: 100vh;
  padding: 24px;
}

.card {
  background-color: var(--surface-color);
  border: 1px solid var(--surface-border);
  border-radius: var(--border-radius);
  padding: 32px;
  width: 100%;
  max-width: 480px;
  box-shadow: 0 10px 25px -5px rgba(0, 0, 0, 0.3);
}

.error-banner {
  background-color: rgba(239, 68, 68, 0.15);
  border: 1px solid var(--destructive);
  color: #fca5a5;
  padding: 10px 14px;
  border-radius: var(--border-radius);
  margin-bottom: 16px;
  font-size: 13px;
  word-break: break-word;
}

.title-1 {
  font-size: 24px;
  font-weight: 700;
  margin-bottom: 8px;
}

.title-2 {
  font-size: 20px;
  font-weight: 600;
}

.dim-label {
  color: var(--text-muted);
  font-size: 13px;
}

.app-layout {
  display: flex;
  flex-direction: column;
  height: 100vh;
}

.app-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 24px;
  background-color: var(--surface-color);
  border-bottom: 1px solid var(--surface-border);
}

.app-content {
  flex: 1;
  padding: 24px;
  overflow-y: auto;
}

.list-box {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-top: 16px;
}

.list-item {
  background-color: var(--surface-color);
  border: 1px solid var(--surface-border);
  border-radius: var(--border-radius);
  padding: 16px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  cursor: pointer;
  transition: border-color 0.15s, background-color 0.15s;
}

.list-item:hover {
  border-color: var(--primary);
  background-color: rgba(59, 130, 246, 0.05);
}

.tabs {
  display: flex;
  gap: 8px;
  border-bottom: 1px solid var(--surface-border);
  margin-bottom: 16px;
}

.tab-btn {
  padding: 8px 16px;
  background: none;
  border: none;
  color: var(--text-muted);
  border-bottom: 2px solid transparent;
  cursor: pointer;
}

.tab-btn.active {
  color: var(--primary);
  border-bottom: 2px solid var(--primary);
  font-weight: 600;
}

.modal-backdrop {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.6);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 1000;
}

.modal {
  background-color: var(--surface-color);
  border: 1px solid var(--surface-border);
  border-radius: var(--border-radius);
  padding: 24px;
  width: 90%;
  max-width: 520px;
}
"#;
