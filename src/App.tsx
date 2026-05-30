import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";

function App() {
  const [ip, setIp] = useState("");
  const [receivedMessage, setReceivedMessage] = useState("");
  const [incomingFile, setIncomingFile] = useState("");
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [senderAccepted, setSenderAccepted] = useState(false);
  const [senderRejected, setSenderRejected] = useState(false);

  useEffect(() => {
    const unlistenMessage = listen<string>(
      "message-received",
      (event) => {
        console.log("Message:", event.payload);
        setReceivedMessage(event.payload);
      }
    );

    const unlistenFileOffer = listen<string>(
      "incoming-file-offer",
      (event) => {
        console.log("File Offer:", event.payload);
        setIncomingFile(event.payload);
      }
    );

    const unlistenAccepted = listen(
      "file-accepted",
      () => {
        alert("Receiver accepted the file");
        setSenderAccepted(true);
      }
    );

    const unlistenRejected = listen(
      "file-rejected",
      () => {
        alert("Receiver rejected the file");
        setSenderRejected(true);
      }
    );

    return () => {
      unlistenMessage.then((fn) => fn());
      unlistenFileOffer.then((fn) => fn());
      unlistenAccepted.then((fn) => fn());
      unlistenRejected.then((fn) => fn());
    };
  }, []);

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

  async function sendFileOffer() {

    if (!selectedFile) {
      alert("Select a file first");
      return;
    }

    try {

      await invoke(
        "send_file_offer",
        {
          ip,
          filename:
            selectedFile.name,

          filesize:
            selectedFile.size,
        }
      );

      console.log(
        "File offer sent"
      );

    } catch (error) {

      console.error(
        "Failed to send file offer:",
        error
      );
    }
  }

  async function acceptFile() {
    try {
      await invoke("send_file_accept", {
        ip,
      });

      setIncomingFile("");
    } catch (error) {
      console.error(error);
    }
  }

  async function rejectFile() {
    try {
      await invoke("send_file_reject", {
        ip,
      });

      setIncomingFile("");
    } catch (error) {
      console.error(error);
    }
  }

  return (
    <div
      style={{
        padding: "40px",
        display: "flex",
        flexDirection: "column",
        gap: "12px",
        maxWidth: "500px",
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

      <input
        type="file"
        onChange={(e) => {
          const file =
            e.target.files?.[0] ?? null;

          setSelectedFile(file);
        }}
      />
      {
        selectedFile && (
          <p>
            Selected:
            {" "}
            {selectedFile.name}
            {" "}
            (
            {selectedFile.size}
            bytes
            )
          </p>
        )
      }
      <button
        onClick={sendFileOffer}
      >
        Send File Offer
      </button>

      <h3>Last Message</h3>

      <p>
        {receivedMessage || "No messages received"}
      </p>

      {senderAccepted && (
        <p>
          Receiver accepted the file
        </p>
      )}

      {senderRejected && (
        <p>
          Receiver rejected the file
        </p>
      )}

      {incomingFile && (
        <div
          style={{
            border: "1px solid #888",
            padding: "12px",
            borderRadius: "8px",
            marginTop: "10px",
          }}
        >
          <h3>Incoming File</h3>

          <p>{incomingFile}</p>

          <div
            style={{
              display: "flex",
              gap: "10px",
            }}
          >
            <button onClick={acceptFile}>
              Accept
            </button>

            <button onClick={rejectFile}>
              Reject
            </button>
          </div>
        </div>
      )}
    </div>
  );
}

export default App;