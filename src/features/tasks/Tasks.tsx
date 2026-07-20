import { useEffect, useMemo, useState } from "react";
import { createTask, deleteTask, listTasks, updateTask } from "../../services/tasks";
import type { Task } from "../../services/tasks";
import { formatUnknownError as formatError } from "../../utils/errors";

const emptyTask = {
  title: "",
  body: "",
  deadline: ""
};

type TaskMode = "idle" | "create" | "view" | "edit";

export function Tasks() {
  const [tasks, setTasks] = useState<Task[]>([]);
  const [selectedId, setSelectedId] = useState<number | null>(null);
  const [taskMode, setTaskMode] = useState<TaskMode>("idle");
  const [form, setForm] = useState(emptyTask);
  const [status, setStatus] = useState("Loading tasks");

  const selectedTask = useMemo(
    () => tasks.find((task) => task.id === selectedId) ?? null,
    [tasks, selectedId]
  );

  useEffect(() => {
    listTasks()
      .then((nextTasks) => {
        setTasks(nextTasks);
        setStatus(nextTasks.length === 0 ? "No tasks yet" : `${nextTasks.length} task(s)`);
      })
      .catch((error) => setStatus(formatError(error)));
  }, []);

  function selectTask(task: Task) {
    setSelectedId(task.id);
    setTaskMode("view");
    setForm({
      title: task.title,
      body: task.body,
      deadline: task.deadline
    });
  }

  function newTask() {
    setSelectedId(null);
    setTaskMode("create");
    setForm(emptyTask);
    setStatus("New task");
  }

  async function onSaveTask() {
    const title = form.title.trim();

    if (!title) {
      setStatus("Task title is required");
      return;
    }

    try {
      if (selectedTask) {
        const updated = await updateTask(selectedTask.id, {
          title,
          body: form.body,
          deadline: form.deadline
        });
        setTasks((current) => current.map((task) => (task.id === updated.id ? updated : task)));
        selectTask(updated);
        setStatus("Task saved");
      } else {
        const created = await createTask(title, form.body, form.deadline);
        setTasks((current) => [created, ...current]);
        selectTask(created);
        setStatus("Task added");
      }
    } catch (error) {
      setStatus(formatError(error));
    }
  }

  async function onDeleteTask() {
    if (!selectedTask) {
      return;
    }

    try {
      await deleteTask(selectedTask.id);
      setTasks((current) => current.filter((task) => task.id !== selectedTask.id));
      setSelectedId(null);
      setTaskMode("idle");
      setForm(emptyTask);
      setStatus("Task deleted");
    } catch (error) {
      setStatus(formatError(error));
    }
  }

  return (
    <section className="feature-panel">
      <div className="panel-heading">
        <div>
          <p>Local Organizer</p>
          <h3>Tasks</h3>
        </div>
        <span className="save-pill">{status}</span>
      </div>

      <div className="split-feature-body">
        <aside className="sub-list" aria-label="Tasks list">
          <button className="primary-button full-width" onClick={newTask} type="button">
            New Task
          </button>
          {tasks.map((task) => (
            <button
              className={task.id === selectedId ? "sub-list-item active" : "sub-list-item"}
              key={task.id}
              onClick={() => selectTask(task)}
              type="button"
            >
              <strong>{task.title}</strong>
              <span>{task.deadline ? formatDeadline(task.deadline) : "No deadline"}</span>
            </button>
          ))}
        </aside>

        {selectedTask || taskMode === "create" ? (
          <form className="editor-form task-editor-form">
            <input
              aria-label="Task title"
              className="text-input"
              readOnly={taskMode === "view"}
              onChange={(event) => setForm((current) => ({ ...current, title: event.target.value }))}
              placeholder="Task title"
              value={form.title}
            />

            <textarea
              aria-label="Task body"
              className="body-input"
              readOnly={taskMode === "view"}
              onChange={(event) => setForm((current) => ({ ...current, body: event.target.value }))}
              placeholder="Task details"
              value={form.body}
            />

            <label className="field-label">
              <span>Deadline</span>
              <input
                aria-label="Task deadline"
                className="text-input"
                readOnly={taskMode === "view"}
                onClick={(event) => {
                  if (taskMode !== "view") {
                    showNativePicker(event.currentTarget);
                  }
                }}
                onFocus={(event) => {
                  if (taskMode !== "view") {
                    showNativePicker(event.currentTarget);
                  }
                }}
                onChange={(event) =>
                  setForm((current) => ({ ...current, deadline: event.target.value }))
                }
                type="datetime-local"
                value={form.deadline}
              />
            </label>

            <div className="form-actions">
              {taskMode === "view" ? (
                <button
                  className="primary-button"
                  onClick={(event) => {
                    event.preventDefault();
                    event.stopPropagation();
                    setTaskMode("edit");
                    setStatus("Editing task");
                  }}
                  type="button"
                >
                  Edit
                </button>
              ) : (
                <button className="primary-button" onClick={() => void onSaveTask()} type="button">
                  {selectedTask ? "Save" : "Add"}
                </button>
              )}
              {selectedTask && (
                <button className="primary-button" onClick={() => void onDeleteTask()} type="button">
                  Delete
                </button>
              )}
            </div>
          </form>
        ) : (
          <div className="empty-editor-state">
            <p>Create or select a task to begin.</p>
          </div>
        )}
      </div>
    </section>
  );
}

function showNativePicker(input: HTMLInputElement) {
  try {
    input.showPicker?.();
  } catch {
    input.focus();
  }
}

function formatDeadline(deadline: string) {
  return deadline.replace("T", " ");
}
