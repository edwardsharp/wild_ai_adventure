/**
 * WebSocket Connection Status Web Component
 *
 * A minimal web component that displays the current WebSocket connection status
 * with a colored indicator (red/yellow/green) and optional text.
 */

import { LitElement, html, css, PropertyValueMap } from 'lit';
import { customElement, property, state } from 'lit/decorators.js';

export enum ConnectionStatus {
  Disconnected = 'disconnected',
  Connecting = 'connecting',
  Connected = 'connected',
  Error = 'error',
}

@customElement('websocket-status')
export class WebSocketStatusComponent extends LitElement {
  static styles = css`
    :host {
      display: inline-flex;
      align-items: center;
      gap: 8px;
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
      font-size: 14px;
    }

    .status-indicator {
      width: 12px;
      height: 12px;
      border-radius: 50%;
      border: 1px solid rgba(0, 0, 0, 0.1);
      transition: all 0.3s ease;
      position: relative;
    }

    .status-indicator.disconnected {
      background-color: #ef4444;
      box-shadow: 0 0 4px rgba(239, 68, 68, 0.3);
    }

    .status-indicator.connecting {
      background-color: #f59e0b;
      box-shadow: 0 0 4px rgba(245, 158, 11, 0.3);
      animation: pulse 1.5s infinite;
    }

    .status-indicator.connected {
      background-color: #10b981;
      box-shadow: 0 0 4px rgba(16, 185, 129, 0.3);
    }

    .status-indicator.error {
      background-color: #dc2626;
      box-shadow: 0 0 4px rgba(220, 38, 38, 0.5);
      animation: blink 1s infinite;
    }

    @keyframes pulse {
      0%, 100% {
        opacity: 1;
        transform: scale(1);
      }
      50% {
        opacity: 0.7;
        transform: scale(1.1);
      }
    }

    @keyframes blink {
      0%, 50% {
        opacity: 1;
      }
      51%, 100% {
        opacity: 0.3;
      }
    }

    .status-text {
      color: #374151;
      font-weight: 500;
    }

    .status-text.disconnected {
      color: #dc2626;
    }

    .status-text.connecting {
      color: #d97706;
    }

    .status-text.connected {
      color: #059669;
    }

    .status-text.error {
      color: #dc2626;
    }

    .user-count {
      color: #6b7280;
      font-size: 12px;
      margin-left: 4px;
    }

    :host([compact]) .status-text {
      display: none;
    }

    :host([compact]) .user-count {
      display: none;
    }
  `;

  @property({ type: String })
  status: ConnectionStatus = ConnectionStatus.Disconnected;

  @property({ type: Boolean })
  showText: boolean = true;

  @property({ type: Number })
  userCount: number = 0;

  @property({ type: Boolean })
  showUserCount: boolean = false;

  @property({ type: Boolean })
  compact: boolean = false;

  @state()
  private lastStatusChange: number = Date.now();

  protected updated(changedProperties: PropertyValueMap<any>): void {
    if (changedProperties.has('status')) {
      this.lastStatusChange = Date.now();
      this.dispatchEvent(new CustomEvent('status-change', {
        detail: {
          status: this.status,
          timestamp: this.lastStatusChange
        },
        bubbles: true
      }));
    }
  }

  private getStatusText(): string {
    switch (this.status) {
      case ConnectionStatus.Disconnected:
        return 'Offline';
      case ConnectionStatus.Connecting:
        return 'Connecting...';
      case ConnectionStatus.Connected:
        return 'Online';
      case ConnectionStatus.Error:
        return 'Connection Error';
      default:
        return 'Unknown';
    }
  }

  render() {
    return html`
      <div class="status-indicator ${this.status}"></div>
      ${this.showText && !this.compact ? html`
        <span class="status-text ${this.status}">
          ${this.getStatusText()}
        </span>
      ` : ''}
      ${this.showUserCount && this.userCount > 0 && !this.compact ? html`
        <span class="user-count">
          (${this.userCount} user${this.userCount !== 1 ? 's' : ''})
        </span>
      ` : ''}
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    'websocket-status': WebSocketStatusComponent;
  }
}
