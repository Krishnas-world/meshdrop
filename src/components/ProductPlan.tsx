const phases = [
  {
    title: "v0.2 Current",
    items: ["Manual LAN IP", "File offer", "Accept or reject", "Save to Downloads/MeshDrop"],
  },
  {
    title: "v0.3 Nearby",
    items: ["Device discovery", "Device names", "Real progress", "Transfer history"],
  },
  {
    title: "v0.4 Power",
    items: ["Chunked transfer", "Folders", "Queue", "Resume"],
  },
  {
    title: "v1.0 Magical",
    items: ["Hotspot mode", "Encryption", "Browser receive", "Multi-device broadcast"],
  },
];

export function ProductPlan() {
  return (
    <aside className="roadmap">
      <div>
        <p className="eyebrow">Product flow</p>
        <h2>Build path</h2>
      </div>

      <div className="roadmap-list">
        {phases.map((phase) => (
          <section className="roadmap-phase" key={phase.title}>
            <h3>{phase.title}</h3>
            <ul>
              {phase.items.map((item) => (
                <li key={item}>{item}</li>
              ))}
            </ul>
          </section>
        ))}
      </div>
    </aside>
  );
}
