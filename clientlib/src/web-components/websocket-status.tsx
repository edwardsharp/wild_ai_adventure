/**
 * WebSocket Connection Status Web Component
 *
 * A minimal web component that displays the current WebSocket connection status
 * with a colored indicator (red/yellow/green) and optional text.
 */

import { customElement } from 'solid-element';
import { createSignal, createEffect, Show } from 'solid-js';
import { ConnectionStatus } from '../lib/websocket-types.js';

export { ConnectionStatus };

export interface WebSocketStatusProps {
  status?: ConnectionStatus;
  showText?: boolean;
  userCount?: number;
  showUserCount?: boolean;
  compact?: boolean;
}

const WebSocketStatus = (props: WebSocketStatusProps) => {
  const [lastStatusChange, setLastStatusChange] = createSignal(Date.now());

  // Create reactive getters with defaults
  const status = () => props.status ?? ConnectionStatus.Disconnected;
  const showText = () => props.showText ?? true;
  const userCount = () => props.userCount ?? 0;
  const showUserCount = () => props.showUserCount ?? false;
  const compact = () => props.compact ?? false;

  // Watch for status changes and dispatch events
  createEffect(() => {
    const currentStatus = status();
    setLastStatusChange(Date.now());

    // Dispatch custom event when status changes
    const event = new CustomEvent('status-change', {
      detail: {
        status: currentStatus,
        timestamp: lastStatusChange(),
      },
      bubbles: true,
    });

    // We'll dispatch this on the host element
    setTimeout(() => {
      const host = document.querySelector('websocket-status');
      if (host) {
        host.dispatchEvent(event);
      }
    }, 0);
  });

  const getStatusText = () => {
    switch (status()) {
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
  };

  const getIndicatorClass = () => `status-indicator ${status()}`;
  const getTextClass = () => `status-text ${status()}`;

  return (
    <div
      style={{
        display: 'inline-flex',
        'align-items': 'center',
        gap: '8px',
        'font-family':
          '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
        'font-size': '14px',
      }}
    >
      <style>{`
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
      `}</style>

      <div class={getIndicatorClass()}></div>

      <Show when={showText() && !compact()}>
        <span class={getTextClass()}>{getStatusText()}</span>
      </Show>

      <Show when={showUserCount() && userCount() > 0 && !compact()}>
        <span class="user-count">
          ({userCount()} user{userCount() !== 1 ? 's' : ''})
        </span>
      </Show>
    </div>
  );
};

// Register as custom element
customElement(
  'websocket-status',
  {
    status: ConnectionStatus.Disconnected,
    showText: true,
    userCount: 0,
    showUserCount: false,
    compact: false,
  },
  WebSocketStatus
);

export { WebSocketStatus };

/* eslint-disable @typescript-eslint/no-namespace */
declare global {
  namespace JSX {
    interface IntrinsicElements {
      'websocket-status': {
        status?: ConnectionStatus;
        showText?: boolean;
        userCount?: number;
        showUserCount?: boolean;
        compact?: boolean;
      };
    }
  }
}
/* eslint-enable @typescript-eslint/no-namespace */
