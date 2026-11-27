import { useEffect, useMemo, useRef, useState, type FormEvent } from "react";
import { ProjectsClient, stages, type Project, type ProjectDraft, type ProjectStage } from "./client";
import "./projects.css";

type View = "Active" | "Waiting on me" | "Archived";

export function ProjectsModule({ sessionId }: { sessionId: string }) {
  const client = useMemo(() => new ProjectsClient(sessionId), [sessionId]);
  const [view, setView] = useState<View>("Active");
  const [search, setSearch] = useState("");
  const [stage, setStage] = useState<ProjectStage | "">("");
  const [revision, setRevision] = useState(0);
  const [projects, setProjects] = useState<Project[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [status, setStatus] = useState("");
  const [busy, setBusy] = useState(false);
  const [editor, setEditor] = useState<{ id: string | null; draft: ProjectDraft }>();
  const [deleting, setDeleting] = useState<Project>();
  const [confirmation, setConfirmation] = useState("");
  useEffect(() => { setStatus(""); }, [view, search, stage]);
  useEffect(() => {
    let active = true;
    setLoading(true);
    const timer = window.setTimeout(() => {
      void client.list({ search, stage: stage || null, archived: view === "Archived", waitingOnly: view === "Waiting on me" })
        .then(items => { if (active) { setProjects(items); setError(""); } })
        .catch(cause => { if (active) { setProjects([]); setError(String(cause)); } })
        .finally(() => { if (active) setLoading(false); });
    }, 150);
    return () => { active = false; window.clearTimeout(timer); };
  }, [client, view, search, stage, revision]);

  async function mutate(action: () => Promise<unknown>, message: string) {
    setBusy(true); setError(""); setStatus("");
    try {
      await action(); setEditor(undefined); setDeleting(undefined); setConfirmation("");
      setStatus(message); setRevision(value => value + 1);
    } catch (cause) { setError(String(cause)); }
    finally { setBusy(false); }
  }

  return <div className="projects-module">
    <header className="row"><div><h1>Projects</h1><p className="muted">Your work, its next step, and what needs you.</p></div>
      <button disabled={busy} onClick={() => { setError(""); setEditor({ id: null, draft: ProjectsClient.empty() }); }}>Add project</button></header>
    <p className="muted">Private to Rob. Links point to existing work. This catalog never changes linked files or repositories.</p>
    <div className="projects-toolbar">
      <div className="actions" role="group" aria-label="Project views">{(["Active", "Waiting on me", "Archived"] as const).map(item =>
        <button key={item} aria-pressed={view === item} onClick={() => setView(item)}>{item}</button>)}</div>
      <label>Search projects<input type="search" value={search} maxLength={2000} onChange={event => setSearch(event.target.value)} placeholder="Title, notes or links" /></label>
      <label>Stage<select value={stage} onChange={event => setStage(event.target.value as ProjectStage | "")}><option value="">All stages</option>{stages.map(item => <option key={item}>{item}</option>)}</select></label>
      <button disabled={busy || loading} onClick={() => setRevision(value => value + 1)}>Refresh</button>
    </div>
    {error && <p className="error" role="alert">{error}</p>}
    <p role="status" className="muted">{loading ? "Loading projects…" : status || `${projects.length} ${projects.length === 1 ? "project" : "projects"}`}</p>
    {!loading && !projects.length && <p className="empty">{search || stage ? "No projects match these filters." : view === "Archived" ? "No archived projects." : view === "Waiting on me" ? "Nothing is waiting on you." : "No active projects."}</p>}
    {!loading && <div className="projects-list">{projects.map(project => <article className="panel project-card" key={project.id} aria-label={project.title}>
      <div className="row"><h2>{project.title}</h2><span className="badge">{project.stage}{project.archived ? " · Archived" : ""}</span></div>
      {project.description && <p>{project.description}</p>}
      <dl><dt>Next step</dt><dd>{project.nextStep || "No next step recorded."}</dd>
        <dt>Waiting on Rob</dt><dd className={project.waitingOnRob ? "project-waiting" : "muted"}>{project.waitingOnRob || "Nothing recorded."}</dd></dl>
      {project.links.length > 0 && <ul className="project-links">{project.links.map((link, index) => <li key={index}><strong>{link.label}</strong><span className="project-target">{link.target}</span>
        {link.target.startsWith("https://") && <button onClick={() => void client.openWebLink(link.target).catch(cause => setError(String(cause)))}>Open web link</button>}</li>)}</ul>}
      <div className="actions"><button disabled={busy} onClick={() => { setError(""); setEditor({ id: project.id, draft: ProjectsClient.draft(project) }); }}>Edit</button>
        <button disabled={busy} onClick={() => void mutate(() => client.archive(project.id, !project.archived), project.archived ? "Project restored." : "Project archived.")}>{project.archived ? "Restore" : "Archive"}</button>
        {project.archived && <button className="project-danger" disabled={busy} onClick={() => { setError(""); setConfirmation(""); setDeleting(project); }}>Permanently delete</button>}
      </div>
    </article>)}</div>}
    {editor && <ProjectEditor key={editor.id ?? "new"} initial={editor.draft} busy={busy} error={error} onCancel={() => { setEditor(undefined); setError(""); }}
      onSave={draft => void mutate(() => client.save(editor.id, draft), "Project saved.")} />}
    {deleting && <ProjectDialog title={`Delete ${deleting.title}?`} busy={busy} onCancel={() => setDeleting(undefined)}>
      <p>This permanently deletes the tracker record only. Linked repositories, directories, files and external projects are untouched.</p>
      <p>Type <strong>{deleting.title}</strong> to confirm.</p>
      <label>Project title to confirm<input autoFocus value={confirmation} onChange={event => setConfirmation(event.target.value)} /></label>
      {error && <p role="alert" className="error">{error}</p>}
      <div className="actions"><button className="project-danger" disabled={busy || confirmation !== deleting.title} onClick={() => void mutate(() => client.delete(deleting.id, confirmation), "Tracker record permanently deleted.")}>Delete tracker record</button>
        <button disabled={busy} onClick={() => setDeleting(undefined)}>Cancel</button></div>
    </ProjectDialog>}
  </div>;
}

function ProjectDialog({ title, busy, onCancel, children }: { title: string; busy: boolean; onCancel: () => void; children: React.ReactNode }) {
  const dialog = useRef<HTMLDialogElement>(null);
  useEffect(() => { dialog.current?.showModal(); }, []);
  return <dialog className="project-dialog projects-module" ref={dialog} aria-label={title} onCancel={event => { event.preventDefault(); if (!busy) onCancel(); }}>
    <h2>{title}</h2>{children}
  </dialog>;
}

function ProjectEditor({ initial, busy, error, onSave, onCancel }: {
  initial: ProjectDraft; busy: boolean; error: string; onSave: (draft: ProjectDraft) => void; onCancel: () => void;
}) {
  const [draft, setDraft] = useState(initial);
  const field = <K extends keyof ProjectDraft>(key: K, value: ProjectDraft[K]) => setDraft(previous => ({ ...previous, [key]: value }));
  const submit = (event: FormEvent) => { event.preventDefault(); onSave(draft); };
  return <ProjectDialog title={initial.title ? "Edit project" : "Add project"} busy={busy} onCancel={onCancel}>
    <form onSubmit={submit}><fieldset disabled={busy}>
      <label>Title<input autoFocus required maxLength={160} value={draft.title} onChange={event => field("title", event.target.value)} /></label>
      <label>Brief description<textarea maxLength={2000} value={draft.description} onChange={event => field("description", event.target.value)} /></label>
      <label>Stage<select value={draft.stage} onChange={event => field("stage", event.target.value as ProjectStage)}>{stages.map(item => <option key={item}>{item}</option>)}</select></label>
      <label>Next step<textarea maxLength={2000} value={draft.nextStep} onChange={event => field("nextStep", event.target.value)} /></label>
      <label>Waiting on Rob<textarea maxLength={2000} placeholder="Leave blank unless there is a concrete action for Rob." value={draft.waitingOnRob} onChange={event => field("waitingOnRob", event.target.value)} /></label>
      <p className="muted">Optional references: HTTPS URLs or absolute workspace/document paths. Paths stay as selectable text and are never opened or executed.</p>
      {draft.links.map((link, index) => <div className="project-link-editor" key={index}>
        <label>Link label<input required maxLength={160} value={link.label} onChange={event => field("links", draft.links.map((item, i) => i === index ? { ...item, label: event.target.value } : item))} /></label>
        <label>URL or absolute path<input required maxLength={2048} value={link.target} onChange={event => field("links", draft.links.map((item, i) => i === index ? { ...item, target: event.target.value } : item))} /></label>
        <button type="button" onClick={() => field("links", draft.links.filter((_, i) => i !== index))}>Remove link</button>
      </div>)}
      <button type="button" disabled={draft.links.length >= 12} onClick={() => field("links", [...draft.links, { label: "", target: "" }])}>Add link</button>
      {error && <p className="error" role="alert">{error}</p>}
      <div className="actions"><button type="submit">{busy ? "Saving…" : "Save project"}</button><button type="button" onClick={onCancel}>Cancel</button></div>
    </fieldset></form>
  </ProjectDialog>;
}
