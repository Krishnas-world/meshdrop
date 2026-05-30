import { invoke } from "@tauri-apps/api/core";
import { useState } from "react";

function App() {
  const [ip, setIp] = useState("");

  async function startServer() {
    try {
      await invoke("start_server");
      console.log("Server started");
    } catch (error) {
      console.error("Failed to start server:", error);
    }
  }

  async function sendMessage() {
    try {
      await invoke("send_message", {
        ip,
        message: "Hello from MeshDrop 🚀",
      });

      console.log("Message sent");
    } catch (error) {
      console.error("Failed to send message:", error);
    }
  }

  return (
    <div
      style={{
        padding: "40px",
        display: "flex",
        flexDirection: "column",
        gap: "12px",
        maxWidth: "400px",
      }}
    >
      <h1>MeshDrop</h1>

      <button onClick={startServer}>
        Start Receiver
      </button>

      <input
        type="text"
        placeholder="Enter receiver IP"
        value={ip}
        onChange={(e) => setIp(e.target.value)}
        style={{
          padding: "10px",
          borderRadius: "8px",
          border: "1px solid #ccc",
        }}
      />

      <button onClick={sendMessage}>
        Send Message
      </button>
    </div>
  );
}

export default App;