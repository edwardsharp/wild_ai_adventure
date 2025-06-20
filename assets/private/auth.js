async function register() {
  let username = document.getElementById("username").value;
  let inviteCode = document.getElementById("invite_code").value;
  if (username === "") {
    alert("Please enter a username");
    return;
  }
  if (inviteCode === "") {
    alert("Please enter an invite or account link code");
    return;
  }

  try {
    // Start registration using the client library
    const challenge = await window.apiClient.registerStart(username, {
      invite_code: inviteCode,
    });

    // Convert challenge data for WebAuthn API
    const credentialCreationOptions = {
      publicKey: {
        ...challenge.publicKey,
        challenge: Base64.toUint8Array(challenge.publicKey.challenge),
        user: {
          ...challenge.publicKey.user,
          id: Base64.toUint8Array(challenge.publicKey.user.id),
        },
        excludeCredentials: challenge.publicKey.excludeCredentials?.map(
          (cred) => ({
            ...cred,
            id: Base64.toUint8Array(cred.id),
          }),
        ),
      },
    };

    // Create credential
    const credential = await navigator.credentials.create(
      credentialCreationOptions,
    );

    if (!credential) {
      throw new Error("Failed to create credential");
    }

    // Finish registration using the client library
    await window.apiClient.registerFinish({
      id: credential.id,
      rawId: Base64.fromUint8Array(new Uint8Array(credential.rawId), true),
      type: credential.type,
      response: {
        attestationObject: Base64.fromUint8Array(
          new Uint8Array(credential.response.attestationObject),
          true,
        ),
        clientDataJSON: Base64.fromUint8Array(
          new Uint8Array(credential.response.clientDataJSON),
          true,
        ),
      },
    });

    const flash_message = document.getElementById("flash_message");
    flash_message.innerHTML = "Successfully registered!";

    // Update auth status after successful registration
    if (window.checkAuthStatus) {
      window.checkAuthStatus();
    }
  } catch (error) {
    const flash_message = document.getElementById("flash_message");
    flash_message.innerHTML = `Registration failed: ${error.message}`;
    console.error("Registration error:", error);
  }
}

async function login() {
  let username = document.getElementById("username").value;
  if (username === "") {
    alert("Please enter a username");
    return;
  }

  try {
    // Start login using the client library
    const challenge = await window.apiClient.loginStart(username);

    // Convert challenge data for WebAuthn API
    const credentialRequestOptions = {
      publicKey: {
        ...challenge.publicKey,
        challenge: Base64.toUint8Array(challenge.publicKey.challenge),
        allowCredentials: challenge.publicKey.allow_credentials?.map(
          (cred) => ({
            ...cred,
            id: Base64.toUint8Array(cred.id),
          }),
        ),
      },
    };

    // Get assertion
    const assertion = await navigator.credentials.get(credentialRequestOptions);

    if (!assertion) {
      throw new Error("Failed to get assertion");
    }

    // Finish login using the client library
    await window.apiClient.loginFinish({
      id: assertion.id,
      rawId: Base64.fromUint8Array(new Uint8Array(assertion.rawId), true),
      type: assertion.type,
      response: {
        authenticatorData: Base64.fromUint8Array(
          new Uint8Array(assertion.response.authenticatorData),
          true,
        ),
        clientDataJSON: Base64.fromUint8Array(
          new Uint8Array(assertion.response.clientDataJSON),
          true,
        ),
        signature: Base64.fromUint8Array(
          new Uint8Array(assertion.response.signature),
          true,
        ),
        userHandle: assertion.response.userHandle
          ? Base64.fromUint8Array(
              new Uint8Array(assertion.response.userHandle),
              true,
            )
          : undefined,
      },
    });

    const flash_message = document.getElementById("flash_message");
    flash_message.innerHTML = "Successfully logged in!";

    // Update auth status after successful login
    if (window.checkAuthStatus) {
      window.checkAuthStatus();
    }
  } catch (error) {
    const flash_message = document.getElementById("flash_message");
    flash_message.innerHTML = `Login failed: ${error.message}`;
    console.error("Login error:", error);
  }
}

async function logout() {
  try {
    // Logout using the client library
    await window.apiClient.logout();

    const flash_message = document.getElementById("flash_message");
    flash_message.innerHTML = "Successfully logged out!";

    // Update auth status after successful logout
    if (window.checkAuthStatus) {
      window.checkAuthStatus();
    }
  } catch (error) {
    const flash_message = document.getElementById("flash_message");
    flash_message.innerHTML = `Logout failed: ${error.message}`;
    console.error("Logout error:", error);
  }
}
