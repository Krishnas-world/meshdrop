import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef, useState } from "react";

function App() {
  const [ip, setIp] = useState("");
  const [receivedMessage, setReceivedMessage] = useState("");
  const [incomingFile, setIncomingFile] = useState("");
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [senderAccepted, setSenderAccepted] = useState(false);
  const [senderRejected, setSenderRejected] = useState(false);
  const [receiveFolder, setReceiveFolder] = useState("");
  const [transferStatus, setTransferStatus] = useState("");
  const selectedFileRef = useRef<File | null>(null);
  const ipRef = useRef("");

  useEffect(() => {
    selectedFileRef.current = selectedFile;
  }, [selectedFile]);

  useEffect(() => {
    ipRef.current = ip;
  }, [ip]);

  useEffect(() => {
    invoke<string>("get_receive_folder")
      .then(setReceiveFolder)
      .catch((error) => console.error("Failed to load receive folder:", error));

    const unlistenMessage = listen<string>("message-received", (event) => {
      console.log("Message:", event.payload);
      setReceivedMessage(event.payload);
    });

    const unlistenFileOffer = listen<string>("incoming-file-offer", (event) => {
      console.log("File Offer:", event.payload);
      setIncomingFile(event.payload);
      setTransferStatus("Incoming file offer");
    });

    const unlistenAccepted = listen("file-accepted", async () => {
      setSenderAccepted(true);
      setSenderRejected(false);

      const file = selectedFileRef.current;
      const targetIp = ipRef.current.trim();

      if (!file || !targetIp) {
        setTransferStatus("Receiver accepted, but no selected file is ready to send.");
        return;
      }

      try {
        setTransferStatus(`Sending ${file.name}...`);
        const bytes = Array.from(new Uint8Array(await file.arrayBuffer()));

        await invoke("send_file_data", {
          ip: targetIp,
          filename: file.name,
          bytes,
        });

        setTransferStatus(`Sent ${file.name}`);
      } catch (error) {
        console.error("Failed to send file data:", error);
        setTransferStatus("Failed to send file data");
      }
    });

    const unlistenRejected = listen("file-rejected", () => {
      setSenderAccepted(false);
      setSenderRejected(true);
      setTransferStatus("Receiver rejected the file");
    });

    const unlistenFileReceived = listen<string>("file-received", (event) => {
      setTransferStatus(`Saved to ${event.payload}`);
    });

    return () => {
      unlistenMessage.then((fn) => fn());
      unlistenFileOffer.then((fn) => fn());
      unlistenAccepted.then((fn) => fn());
      unlistenRejected.then((fn) => fn());
      unlistenFileReceived.then((fn) => fn());
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
        message: "Hello from MeshDrop",
      });

      console.log("Message sent");
    } catch (error) {
      console.error("Failed to send message:", error);
    }
  }

  async function sendFileOffer() {
    if (!selectedFile) {
      setTransferStatus("Select a file first");
      return;
    }

    try {
      await invoke("send_file_offer", {
        ip,
        filename: selectedFile.name,
        filesize: selectedFile.size,
      });

      console.log("File offer sent");
      setTransferStatus(`Offer sent for ${selectedFile.name}`);
      setSenderAccepted(false);
      setSenderRejected(false);
    } catch (error) {
      console.error("Failed to send file offer:", error);
      setTransferStatus("Failed to send file offer");
    }
  }

  async function acceptFile() {
    try {
      const savedFolder = await invoke<string>("set_receive_folder", {
        path: receiveFolder,
      });

      setReceiveFolder(savedFolder);

      await invoke("send_file_accept", {
        ip,
      });

      setIncomingFile("");
      setTransferStatus("Accepted. Waiting for file data...");
    } catch (error) {
      console.error(error);
      setTransferStatus("Could not accept file. Check the receive folder path.");
    }
  }

  async function rejectFile() {
    try {
      await invoke("send_file_reject", {
        ip,
      });

      setIncomingFile("");
      setTransferStatus("File rejected");
    } catch (error) {
      console.error(error);
    }
  }

  const [incomingName, incomingSize] = incomingFile.split("|");

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

      <button onClick={startServer}>Start Receiver</button>

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

      <button onClick={sendMessage}>Send Message</button>

      <label>
        Receive folder
        <input
          type="text"
          value={receiveFolder}
          onChange={(e) => setReceiveFolder(e.target.value)}
          style={{
            boxSizing: "border-box",
            display: "block",
            marginTop: "6px",
            width: "100%",
            padding: "10px",
            borderRadius: "8px",
            border: "1px solid #ccc",
          }}
        />
      </label>

      <input
        type="file"
        onChange={(e) => {
          const file = e.target.files?.[0] ?? null;
          setSelectedFile(file);
        }}
      />

      {selectedFile && (
        <p>
          Selected: {selectedFile.name} ({selectedFile.size} bytes)
        </p>
      )}

      <button onClick={sendFileOffer}>Send File Offer</button>

      <h3>Last Message</h3>

      <p>{receivedMessage || "No messages received"}</p>

      {senderAccepted && <p>Receiver accepted. Sending file data now.</p>}

      {senderRejected && <p>Receiver rejected the file</p>}

      {transferStatus && <p>{transferStatus}</p>}

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

          <p>
            {incomingName} ({incomingSize} bytes)
          </p>

          <div
            style={{
              display: "flex",
              gap: "10px",
            }}
          >
            <button onClick={acceptFile}>Accept</button>

            <button onClick={rejectFile}>Reject</button>
          </div>
        </div>
      )}
    </div>
  );
}

export default App;
