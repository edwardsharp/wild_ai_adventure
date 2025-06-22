import { customElement } from 'solid-element';
import {
  createSignal,
  createEffect,
  onMount,
  Show,
  createMemo,
} from 'solid-js';

// Import the API client from parent clientlib
import { ApiClient, ApiError } from '@webauthn/clientlib';

// WebAuthn types
type UserVerificationRequirement = 'required' | 'preferred' | 'discouraged';

export interface WebAuthnAuthProps {
  baseUrl?: string;
  theme?: 'light' | 'dark' | 'auto';
}

// Base64 utility functions for WebAuthn
const base64ToUint8Array = (base64: string): Uint8Array => {
  const binaryString = atob(base64.replace(/-/g, '+').replace(/_/g, '/'));
  const bytes = new Uint8Array(binaryString.length);
  for (let i = 0; i < binaryString.length; i++) {
    bytes[i] = binaryString.charCodeAt(i);
  }
  return bytes;
};

const uint8ArrayToBase64 = (uint8Array: Uint8Array): string => {
  let binaryString = '';
  for (let i = 0; i < uint8Array.length; i++) {
    binaryString += String.fromCharCode(uint8Array[i]!);
  }
  return btoa(binaryString)
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=/g, '');
};

customElement('webauthn-auth', { baseUrl: '', theme: 'auto' }, (props) => {
  const [username, setUsername] = createSignal('');
  const [inviteCode, setInviteCode] = createSignal('');
  const [isAuthenticated, setIsAuthenticated] = createSignal(false);
  const [currentUser, setCurrentUser] = createSignal<string | null>(null);
  const [isLoading, setIsLoading] = createSignal(false);
  const [message, setMessage] = createSignal('');
  const [messageType, setMessageType] = createSignal<
    'info' | 'success' | 'error'
  >('info');
  const [mode, setMode] = createSignal<'login' | 'register'>('login');

  const apiClient = createMemo(
    () =>
      new ApiClient({
        baseUrl: props.baseUrl || 'http://localhost:8080',
      })
  );

  const showMessage = (
    msg: string,
    type: 'info' | 'success' | 'error' = 'info'
  ) => {
    setMessage(msg);
    setMessageType(type);
    setTimeout(() => setMessage(''), 5000);
  };

  const handleError = (error: unknown) => {
    const errorMessage =
      error instanceof ApiError
        ? `${error.message} (${error.status})`
        : error instanceof Error
          ? error.message
          : 'An unknown error occurred';

    showMessage(errorMessage, 'error');

    // Dispatch custom event for error
    const element = document.querySelector('webauthn-auth');
    if (element) {
      element.dispatchEvent(
        new CustomEvent('webauthn-error', {
          detail: { error: errorMessage },
          bubbles: true,
        })
      );
    }
  };

  const checkAuthStatus = async () => {
    try {
      const status = await apiClient().authStatus();
      setIsAuthenticated(status.authenticated);
      setCurrentUser(status.username || null);
      return status.authenticated;
    } catch {
      setIsAuthenticated(false);
      setCurrentUser(null);
      return false;
    }
  };

  const handleRegister = async () => {
    if (!username() || !inviteCode()) {
      showMessage('Please enter both username and invite code', 'error');
      return;
    }

    setIsLoading(true);
    try {
      // Start registration
      const challenge = await apiClient().registerStart(username(), {
        invite_code: inviteCode(),
      });

      // Convert challenge data for WebAuthn API
      const credentialCreationOptions: CredentialCreationOptions = {
        publicKey: {
          ...challenge.publicKey,
          challenge: base64ToUint8Array(challenge.publicKey.challenge),
          attestation: challenge.publicKey
            .attestation as AttestationConveyancePreference,
          user: {
            ...challenge.publicKey.user,
            id: base64ToUint8Array(challenge.publicKey.user.id),
          },
          authenticatorSelection: {
            ...challenge.publicKey.authenticatorSelection,
            residentKey: challenge.publicKey.authenticatorSelection
              .residentKey as ResidentKeyRequirement,
            userVerification: challenge.publicKey.authenticatorSelection
              .userVerification as UserVerificationRequirement,
          },
          excludeCredentials: challenge.publicKey.excludeCredentials?.map(
            (cred) => ({
              ...cred,
              id: base64ToUint8Array(cred.id),
            })
          ),
          ...(challenge.publicKey.extensions && {
            extensions: challenge.publicKey.extensions,
          }),
        },
      };

      // Create credential
      const credential = (await navigator.credentials.create(
        credentialCreationOptions
      )) as PublicKeyCredential;

      if (!credential) {
        throw new Error('Failed to create credential');
      }

      // Finish registration
      await apiClient().registerFinish({
        id: credential.id,
        rawId: uint8ArrayToBase64(new Uint8Array(credential.rawId)),
        type: credential.type,
        response: {
          attestationObject: uint8ArrayToBase64(
            new Uint8Array(
              (
                credential.response as AuthenticatorAttestationResponse
              ).attestationObject
            )
          ),
          clientDataJSON: uint8ArrayToBase64(
            new Uint8Array(credential.response.clientDataJSON)
          ),
        },
      });

      showMessage('Registration successful!', 'success');
      await checkAuthStatus();

      // Dispatch custom event for login
      const element = document.querySelector('webauthn-auth');
      if (element) {
        element.dispatchEvent(
          new CustomEvent('webauthn-login', {
            detail: { username: username() },
            bubbles: true,
          })
        );
      }
    } catch (error) {
      handleError(error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleLogin = async () => {
    if (!username()) {
      showMessage('Please enter a username', 'error');
      return;
    }

    setIsLoading(true);
    try {
      // Start login
      const challenge = await apiClient().loginStart(username());

      // Convert challenge data for WebAuthn API
      const credentialRequestOptions: CredentialRequestOptions = {
        publicKey: {
          ...challenge.publicKey,
          challenge: base64ToUint8Array(challenge.publicKey.challenge),
          userVerification: challenge.publicKey
            .userVerification as UserVerificationRequirement,
          allowCredentials: challenge.publicKey.allowCredentials?.map(
            (cred) => ({
              ...cred,
              id: base64ToUint8Array(cred.id),
            })
          ),
        },
      };

      // Get assertion
      const assertion = (await navigator.credentials.get(
        credentialRequestOptions
      )) as PublicKeyCredential;

      if (!assertion) {
        throw new Error('Failed to get assertion');
      }

      // Finish login
      await apiClient().loginFinish({
        id: assertion.id,
        rawId: uint8ArrayToBase64(new Uint8Array(assertion.rawId)),
        type: assertion.type,
        response: {
          authenticatorData: uint8ArrayToBase64(
            new Uint8Array(
              (
                assertion.response as AuthenticatorAssertionResponse
              ).authenticatorData
            )
          ),
          clientDataJSON: uint8ArrayToBase64(
            new Uint8Array(assertion.response.clientDataJSON)
          ),
          signature: uint8ArrayToBase64(
            new Uint8Array(
              (assertion.response as AuthenticatorAssertionResponse).signature
            )
          ),
          userHandle: (assertion.response as AuthenticatorAssertionResponse)
            .userHandle
            ? uint8ArrayToBase64(
                new Uint8Array(
                  (
                    assertion.response as AuthenticatorAssertionResponse
                  ).userHandle!
                )
              )
            : undefined,
        },
      });

      showMessage('Login successful!', 'success');
      await checkAuthStatus();

      // Dispatch custom event for login
      const element = document.querySelector('webauthn-auth');
      if (element) {
        element.dispatchEvent(
          new CustomEvent('webauthn-login', {
            detail: { username: username() },
            bubbles: true,
          })
        );
      }
    } catch (error) {
      handleError(error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleLogout = async () => {
    setIsLoading(true);
    try {
      await apiClient().logout();
      setIsAuthenticated(false);
      setCurrentUser(null);
      setUsername('');
      setInviteCode('');
      showMessage('Logged out successfully', 'success');

      // Dispatch custom event for logout
      const element = document.querySelector('webauthn-auth');
      if (element) {
        element.dispatchEvent(
          new CustomEvent('webauthn-logout', {
            bubbles: true,
          })
        );
      }
    } catch (error) {
      handleError(error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleEnterKey = (event: KeyboardEvent) => {
    if (event.key === 'Enter') {
      if (mode() === 'register' && username() && inviteCode()) {
        handleRegister();
      } else if (mode() === 'login' && username()) {
        handleLogin();
      }
    }
  };

  const updateMode = () => {
    if (username() && inviteCode()) {
      setMode('register');
    } else if (username()) {
      setMode('login');
    }
  };

  // Check auth status on mount
  onMount(() => {
    checkAuthStatus();
  });

  // Update mode when inputs change
  createEffect(() => {
    updateMode();
  });

  const getThemeClass = () => {
    const theme = props.theme || 'auto';
    if (theme === 'auto') {
      return 'webauthn-theme-auto';
    }
    return `webauthn-theme-${theme}`;
  };

  return (
    <div
      class={`webauthn-auth ${getThemeClass()}`}
      style={{
        'max-width': '400px',
        margin: '0 auto',
        padding: '1.5rem',
        border: '1px solid #e1e5e9',
        'border-radius': '8px',
        background: props.theme === 'dark' ? '#1a1a1a' : 'white',
        color: props.theme === 'dark' ? 'white' : 'black',
        'font-family':
          '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
      }}
    >
      <h2
        style={{
          'text-align': 'center',
          'margin-bottom': '1.5rem',
          'font-size': '1.5rem',
          'font-weight': '600',
        }}
      >
        WebAuthn Authentication
      </h2>

      <Show when={message()}>
        <div
          style={{
            padding: '0.75rem',
            'border-radius': '6px',
            'margin-bottom': '1rem',
            'font-size': '0.875rem',
            background:
              messageType() === 'error'
                ? '#fee2e2'
                : messageType() === 'success'
                  ? '#dcfce7'
                  : '#dbeafe',
            color:
              messageType() === 'error'
                ? '#991b1b'
                : messageType() === 'success'
                  ? '#166534'
                  : '#1e40af',
          }}
        >
          {message()}
        </div>
      </Show>

      <Show
        when={!isAuthenticated()}
        fallback={
          <div>
            <div style={{ 'text-align': 'center', 'margin-bottom': '1rem' }}>
              <p>
                Welcome,{' '}
                <span style={{ 'font-weight': '600' }}>{currentUser()}</span>!
              </p>
              <p
                style={{
                  'font-size': '0.875rem',
                  color: '#6b7280',
                  'text-align': 'center',
                  'margin-top': '0.5rem',
                }}
              >
                You are successfully authenticated.
              </p>
            </div>
            <button
              style={{
                padding: '0.75rem 1.5rem',
                border: 'none',
                'border-radius': '6px',
                'font-size': '1rem',
                'font-weight': '500',
                cursor: isLoading() ? 'not-allowed' : 'pointer',
                transition: 'all 0.2s',
                'min-height': '48px',
                display: 'flex',
                'align-items': 'center',
                'justify-content': 'center',
                gap: '0.5rem',
                background: '#ef4444',
                color: 'white',
              }}
              onClick={handleLogout}
              disabled={isLoading()}
            >
              <Show when={isLoading()}>
                <div
                  style={{
                    width: '20px',
                    height: '20px',
                    border: '2px solid transparent',
                    'border-top': '2px solid currentColor',
                    'border-radius': '50%',
                    animation: 'spin 1s linear infinite',
                  }}
                />
              </Show>
              Logout
            </button>
          </div>
        }
      >
        <div
          style={{ display: 'flex', 'flex-direction': 'column', gap: '1rem' }}
        >
          <input
            style={{
              padding: '0.75rem',
              border: '1px solid #d1d5db',
              'border-radius': '6px',
              'font-size': '1rem',
              background: props.theme === 'dark' ? '#2a2a2a' : 'white',
              color: props.theme === 'dark' ? 'white' : 'black',
            }}
            type='text'
            placeholder='Username'
            value={username()}
            onInput={(e) => setUsername(e.currentTarget.value)}
            onKeyDown={handleEnterKey}
            disabled={isLoading()}
          />

          <input
            style={{
              padding: '0.75rem',
              border: '1px solid #d1d5db',
              'border-radius': '6px',
              'font-size': '1rem',
              background: props.theme === 'dark' ? '#2a2a2a' : 'white',
              color: props.theme === 'dark' ? 'white' : 'black',
            }}
            type='text'
            placeholder='Invite or account link code (optional)'
            value={inviteCode()}
            onInput={(e) => setInviteCode(e.currentTarget.value)}
            onKeyDown={handleEnterKey}
            disabled={isLoading()}
          />

          <Show when={mode() === 'register'}>
            <button
              style={{
                padding: '0.75rem 1.5rem',
                border: 'none',
                'border-radius': '6px',
                'font-size': '1rem',
                'font-weight': '500',
                cursor:
                  isLoading() || !username() || !inviteCode()
                    ? 'not-allowed'
                    : 'pointer',
                transition: 'all 0.2s',
                'min-height': '48px',
                display: 'flex',
                'align-items': 'center',
                'justify-content': 'center',
                gap: '0.5rem',
                background:
                  isLoading() || !username() || !inviteCode()
                    ? '#9ca3af'
                    : '#10b981',
                color: 'white',
              }}
              onClick={handleRegister}
              disabled={isLoading() || !username() || !inviteCode()}
            >
              <Show when={isLoading()}>
                <div
                  style={{
                    width: '20px',
                    height: '20px',
                    border: '2px solid transparent',
                    'border-top': '2px solid currentColor',
                    'border-radius': '50%',
                    animation: 'spin 1s linear infinite',
                  }}
                />
              </Show>
              Register
            </button>
            <p
              style={{
                'font-size': '0.875rem',
                color: '#6b7280',
                'text-align': 'center',
                'margin-top': '0.5rem',
              }}
            >
              Ready to register with invite code (Press Enter)
            </p>
          </Show>

          <Show when={mode() === 'login'}>
            <button
              style={{
                padding: '0.75rem 1.5rem',
                border: 'none',
                'border-radius': '6px',
                'font-size': '1rem',
                'font-weight': '500',
                cursor: isLoading() || !username() ? 'not-allowed' : 'pointer',
                transition: 'all 0.2s',
                'min-height': '48px',
                display: 'flex',
                'align-items': 'center',
                'justify-content': 'center',
                gap: '0.5rem',
                background: isLoading() || !username() ? '#9ca3af' : '#3b82f6',
                color: 'white',
              }}
              onClick={handleLogin}
              disabled={isLoading() || !username()}
            >
              <Show when={isLoading()}>
                <div
                  style={{
                    width: '20px',
                    height: '20px',
                    border: '2px solid transparent',
                    'border-top': '2px solid currentColor',
                    'border-radius': '50%',
                    animation: 'spin 1s linear infinite',
                  }}
                />
              </Show>
              Login
            </button>
            <p
              style={{
                'font-size': '0.875rem',
                color: '#6b7280',
                'text-align': 'center',
                'margin-top': '0.5rem',
              }}
            >
              {username()
                ? 'Ready to login (Press Enter, or add invite code to register)'
                : 'Enter your username to login, or username + invite code to register'}
            </p>
          </Show>
        </div>
      </Show>
    </div>
  );
});

// Export version info
export const VERSION = '1.0.0';
