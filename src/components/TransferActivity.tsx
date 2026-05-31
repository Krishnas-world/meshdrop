import type { ActivityItem } from "../types";

type TransferActivityProps = {
  progress: number;
  status: string;
  receivedMessage: string;
  activity: ActivityItem[];
};

export function TransferActivity({
  progress,
  status,
  receivedMessage,
  activity,
}: TransferActivityProps) {
  return (
    <section className="panel activity-panel">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">Live</p>
          <h2>Transfer queue</h2>
        </div>
        <span className="progress-value">{progress}%</span>
      </div>

      <div className="progress-track" aria-label="Transfer progress">
        <span style={{ width: `${progress}%` }} />
      </div>

      <p className="status-copy">{status}</p>

      <div className="message-preview">
        <span>Last message</span>
        <strong>{receivedMessage || "No messages received"}</strong>
      </div>

      <div className="activity-list">
        {activity.length === 0 ? (
          <p className="empty-state">Transfers and messages will appear here.</p>
        ) : (
          activity.slice(0, 5).map((item) => (
            <article className="activity-item" key={item.id}>
              <span className={`activity-dot activity-dot-${item.status}`} />
              <div>
                <strong>{item.title}</strong>
                <small>{item.detail}</small>
              </div>
              <time>{item.time}</time>
            </article>
          ))
        )}
      </div>
    </section>
  );
}
