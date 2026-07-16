interface Props {
  title: string;
  phase: string;
  blurb: string;
}

export default function Placeholder({ title, phase, blurb }: Props) {
  return (
    <>
      <div className="topbar">
        <h1>{title}</h1>
      </div>
      <div className="panel placeholder">
        <span className="phase">{phase}</span>
        <h3>{title} is on the way</h3>
        <p>{blurb}</p>
      </div>
    </>
  );
}
