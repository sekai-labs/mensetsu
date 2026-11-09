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

.settings-header-left {
  display: flex;
  align-items: center;
  gap: 14px;
  min-width: 0;
}

.settings-header-title {
  min-width: 0;
}

.settings-page-title {
  font-size: 18px;
  line-height: 1.3;
}

.settings-shell {
  display: grid;
  grid-template-columns: 168px minmax(0, 1fr);
  gap: 24px;
  align-items: start;
  max-width: 980px;
  margin: 0 auto;
}

.settings-nav {
  position: sticky;
  top: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
  padding-top: 2px;
}

.settings-nav-label {
  padding: 6px 10px;
  color: var(--text-muted);
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.settings-nav-link {
  display: block;
  padding: 6px 10px;
  border-left: 2px solid transparent;
  border-radius: 0 4px 4px 0;
  color: var(--text-muted);
  font-size: 12px;
  text-decoration: none;
}

.settings-nav-link:hover {
  color: var(--text-main);
  background-color: rgba(148, 163, 184, 0.08);
  border-left-color: var(--surface-border);
}

.settings-nav-link:focus-visible {
  outline: 2px solid var(--primary);
  outline-offset: 1px;
}

.settings-sections {
  display: flex;
  flex-direction: column;
  gap: 16px;
  min-width: 0;
}

.settings-card {
  min-width: 0;
  padding: 20px;
  background-color: var(--surface-color);
  border: 1px solid var(--surface-border);
  border-radius: var(--border-radius);
  scroll-margin-top: 12px;
}

.settings-card-head {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 16px;
  margin-bottom: 14px;
}

.settings-title {
  font-size: 15px;
  font-weight: 600;
  line-height: 1.35;
}

.settings-copy {
  margin-top: 2px;
  color: var(--text-muted);
  font-size: 12px;
}

.settings-form-grid {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1.4fr);
  gap: 12px;
}

.field-label {
  display: block;
  margin-bottom: 5px;
  color: var(--text-muted);
  font-size: 12px;
  font-weight: 500;
}

.field-help {
  margin-top: 6px;
  color: var(--text-muted);
  font-size: 11px;
}

.settings-actions {
  display: flex;
  justify-content: flex-end;
  margin-top: 14px;
}

.button-compact {
  padding: 5px 10px;
  font-size: 12px;
}

.settings-subsection {
  margin-top: 20px;
  padding-top: 16px;
  border-top: 1px solid var(--surface-border);
}

.settings-subsection-head {
  margin-bottom: 10px;
}

.settings-subsection-title {
  font-size: 13px;
  font-weight: 600;
}

.settings-subsection-copy {
  margin-top: 2px;
  color: var(--text-muted);
  font-size: 11px;
}

.provider-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  min-width: 0;
}

.provider-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  min-width: 0;
  padding: 10px 12px;
  background-color: var(--bg-color);
  border: 1px solid var(--surface-border);
  border-radius: 6px;
}

.provider-identity {
  min-width: 0;
}

.provider-name {
  display: block;
  font-size: 13px;
  font-weight: 600;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.provider-meta {
  display: block;
  margin-top: 1px;
  color: var(--text-muted);
  font-size: 11px;
}

.provider-endpoint {
  display: block;
  margin-top: 1px;
  max-width: 100%;
  color: var(--text-muted);
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 11px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.provider-badges {
  display: flex;
  flex-shrink: 0;
  align-items: center;
  gap: 6px;
}

.badge {
  display: inline-flex;
  align-items: center;
  padding: 2px 8px;
  border: 1px solid transparent;
  border-radius: 999px;
  font-size: 10px;
  font-weight: 600;
  line-height: 1.6;
  white-space: nowrap;
}

.badge-success {
  color: var(--success);
  background-color: rgba(16, 185, 129, 0.12);
  border-color: rgba(16, 185, 129, 0.3);
}

.badge-neutral {
  color: var(--text-muted);
  background-color: rgba(148, 163, 184, 0.1);
  border-color: var(--surface-border);
}

.empty-state {
  padding: 16px;
  border: 1px dashed var(--surface-border);
  border-radius: 6px;
  color: var(--text-muted);
  font-size: 12px;
  text-align: center;
}

.usage-metrics {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 1px;
  overflow: hidden;
  background-color: var(--surface-border);
  border: 1px solid var(--surface-border);
  border-radius: 6px;
}

.usage-metric {
  min-width: 0;
  padding: 10px 12px;
  background-color: var(--bg-color);
}

.usage-metric-label {
  color: var(--text-muted);
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.usage-metric-value {
  margin-top: 3px;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 16px;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
  white-space: nowrap;
}

.usage-metric-value-accent {
  color: var(--primary);
}

.usage-metrics-skeleton .usage-metric {
  min-height: 56px;
}

.skeleton {
  background-color: var(--surface-border);
  border-radius: 4px;
  opacity: 0.55;
}

.skeleton-metric {
  width: 64%;
  height: 14px;
  margin-top: 2px;
}

.skeleton-row {
  height: 28px;
  margin-top: 8px;
}

.usage-loading,
.usage-refreshing {
  margin-top: 12px;
}

.usage-empty {
  margin-top: 12px;
  padding: 18px 14px;
  border: 1px dashed var(--surface-border);
  border-radius: 6px;
  color: var(--text-muted);
  font-size: 12px;
  text-align: center;
}

.usage-table-scroll {
  max-width: 100%;
  max-height: 340px;
  margin-top: 12px;
  overflow: auto;
  border: 1px solid var(--surface-border);
  border-radius: 6px;
}

.usage-table {
  width: 100%;
  min-width: 660px;
  border-collapse: collapse;
  font-size: 12px;
  text-align: left;
}

.usage-table th {
  position: sticky;
  top: 0;
  z-index: 1;
  padding: 7px 10px;
  background-color: var(--bg-color);
  border-bottom: 1px solid var(--surface-border);
  color: var(--text-muted);
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.05em;
  text-transform: uppercase;
  white-space: nowrap;
}

.usage-table td {
  padding: 7px 10px;
  border-bottom: 1px solid var(--surface-border);
  vertical-align: middle;
}

.usage-table tbody tr:last-child td {
  border-bottom: none;
}

.usage-table tbody tr:hover {
  background-color: rgba(148, 163, 184, 0.06);
}

.usage-time {
  color: var(--text-muted);
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 11px;
  white-space: nowrap;
}

.usage-model {
  color: var(--text-main);
  font-weight: 500;
}

.usage-provider {
  display: block;
  margin-top: 1px;
  color: var(--text-muted);
  font-size: 10px;
}

.usage-agent {
  max-width: 180px;
  color: var(--text-muted);
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 11px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.usage-number {
  color: var(--text-main);
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-variant-numeric: tabular-nums;
  text-align: right;
  white-space: nowrap;
}

.usage-number-total {
  font-weight: 600;
}

.usage-footer {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  margin-top: 10px;
}

.usage-footer-note {
  color: var(--text-muted);
  font-size: 11px;
}

.inline-error {
  margin-top: 8px;
  color: var(--destructive);
  font-size: 11px;
}

.visually-hidden {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  border: 0;
}

@media (max-width: 900px) {
  .settings-shell {
    grid-template-columns: minmax(0, 1fr);
    gap: 12px;
  }

  .settings-nav {
    position: static;
    flex-direction: row;
    gap: 4px;
    overflow-x: auto;
    padding-bottom: 2px;
  }

  .settings-nav-label {
    display: none;
  }

  .settings-nav-link {
    border-left: none;
    border-bottom: 2px solid transparent;
    border-radius: 4px 4px 0 0;
    white-space: nowrap;
  }

  .settings-nav-link:hover {
    border-bottom-color: var(--surface-border);
  }
}

@media (max-width: 720px) {
  .settings-form-grid {
    grid-template-columns: minmax(0, 1fr);
  }

  .usage-metrics {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .settings-card {
    padding: 16px;
  }
}
"#;
