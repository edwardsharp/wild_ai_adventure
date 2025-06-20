function register() {
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

  fetch(
    "http://localhost:8080/register_start/" +
      encodeURIComponent(username) +
      "?invite_code=" +
      encodeURIComponent(inviteCode),
    {
      method: "POST",
    },
  )
    .then((response) => response.json())
    .then((credentialCreationOptions) => {
      credentialCreationOptions.publicKey.challenge = Base64.toUint8Array(
        credentialCreationOptions.publicKey.challenge,
      );
      credentialCreationOptions.publicKey.user.id = Base64.toUint8Array(
        credentialCreationOptions.publicKey.user.id,
      );
      credentialCreationOptions.publicKey.excludeCredentials?.forEach(
        function (listItem) {
          listItem.id = Base64.toUint8Array(listItem.id);
        },
      );

      return navigator.credentials.create({
        publicKey: credentialCreationOptions.publicKey,
      });
    })
    .then((credential) => {
      fetch("http://localhost:8080/register_finish", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
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
        }),
      }).then((response) => {
        const flash_message = document.getElementById("flash_message");
        if (response.ok) {
          response
            .json()
            .then((data) => {
              // Use the message from the backend response
              flash_message.innerHTML = data.message;
              // Update auth status after successful registration
              if (window.checkAuthStatus) {
                window.checkAuthStatus();
              }
            })
            .catch(() => {
              flash_message.innerHTML = "Successfully registered!";
              // Update auth status after successful registration
              if (window.checkAuthStatus) {
                window.checkAuthStatus();
              }
            });
        } else {
          response
            .text()
            .then((errorMsg) => {
              flash_message.innerHTML = `Registration failed: ${errorMsg}`;
            })
            .catch(() => {
              flash_message.innerHTML = "Error whilst registering!";
            });
        }
      });
    });
}

function login() {
  let username = document.getElementById("username").value;
  if (username === "") {
    alert("Please enter a username");
    return;
  }

  fetch("http://localhost:8080/login_start/" + encodeURIComponent(username), {
    method: "POST",
  })
    .then((response) => response.json())
    .then((credentialRequestOptions) => {
      credentialRequestOptions.publicKey.challenge = Base64.toUint8Array(
        credentialRequestOptions.publicKey.challenge,
      );
      credentialRequestOptions.publicKey.allowCredentials?.forEach(
        function (listItem) {
          listItem.id = Base64.toUint8Array(listItem.id);
        },
      );

      return navigator.credentials.get({
        publicKey: credentialRequestOptions.publicKey,
      });
    })
    .then((assertion) => {
      fetch("http://localhost:8080/login_finish", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
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
            userHandle: Base64.fromUint8Array(
              new Uint8Array(assertion.response.userHandle),
              true,
            ),
          },
        }),
      }).then((response) => {
        const flash_message = document.getElementById("flash_message");
        if (response.ok) {
          flash_message.innerHTML = "Successfully logged in!";
          // Update auth status after successful login
          if (window.checkAuthStatus) {
            window.checkAuthStatus();
          }
        } else {
          flash_message.innerHTML = "Error whilst logging in!";
        }
      });
    });
}

function logout() {
  fetch("http://localhost:8080/logout", {
    method: "POST",
  })
    .then((response) => {
      const flash_message = document.getElementById("flash_message");
      if (response.ok) {
        flash_message.innerHTML = "Successfully logged out!";
        // Update auth status after successful logout
        if (window.checkAuthStatus) {
          window.checkAuthStatus();
        }
      } else {
        flash_message.innerHTML = "Error whilst logging out!";
      }
    })
    .catch((error) => {
      const flash_message = document.getElementById("flash_message");
      flash_message.innerHTML = "Error whilst logging out: " + error;
    });
}
