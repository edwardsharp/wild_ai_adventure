/* @jsxImportSource solid-js */
import { render } from "solid-js/web";
import { createSignal } from "solid-js";

console.log("🚀 Script started loading");

function SimpleTest() {
  console.log("📦 SimpleTest component created");
  const [count, setCount] = createSignal(0);

  return (
    <div style={{ padding: "20px", border: "1px solid #ccc", margin: "20px" }}>
      <h2>Simple Solid.js Test</h2>
      <p>Count: {count()}</p>
      <button onClick={() => setCount(count() + 1)}>Increment</button>
    </div>
  );
}

// Create a simple custom element manually without solid-element
class SimpleTestElement extends HTMLElement {
  private dispose?: () => void;

  connectedCallback() {
    console.log("🔌 SimpleTestElement connected");
    try {
      this.dispose = render(() => <SimpleTest />, this);
      console.log("✅ Render successful");
    } catch (error) {
      console.error("❌ Render failed:", error);
    }
  }

  disconnectedCallback() {
    console.log("🔌 SimpleTestElement disconnected");
    if (this.dispose) {
      this.dispose();
    }
  }
}

console.log("📝 About to register custom element");

try {
  // Register the custom element
  customElements.define("simple-test", SimpleTestElement);
  console.log("✅ Custom element registered successfully");
} catch (error) {
  console.error("❌ Failed to register custom element:", error);
}

export { SimpleTest, SimpleTestElement };
