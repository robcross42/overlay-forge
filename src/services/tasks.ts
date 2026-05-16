import { invoke } from "@tauri-apps/api/core";

export type Task = {
  id: number;
  title: string;
  body: string;
  deadline: string;
  isCompleted: boolean;
  createdAt: string;
  updatedAt: string;
};

export function listTasks() {
  return invoke<Task[]>("list_tasks");
}

export function createTask(title: string, body: string, deadline: string) {
  return invoke<Task>("create_task", { title, body, deadline });
}

export function updateTask(
  id: number,
  values: { title?: string; body?: string; deadline?: string; isCompleted?: boolean }
) {
  return invoke<Task>("update_task", { id, ...values });
}

export function deleteTask(id: number) {
  return invoke<void>("delete_task", { id });
}
