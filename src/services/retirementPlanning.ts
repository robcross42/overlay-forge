import { invoke } from "@tauri-apps/api/core";

export type ProtectedStoreStatus = { state: "uninitialized" | "locked" | "unlocked"; message: string };
export type RetirementProfile = { displayLabel: string; age: number | null; targetAge: number | null; retirementDefinition: string; notes: string };
export type RetirementFinancialRecord = { id: string; entityType: "account" | "debt" | "income"; kind: string; label: string; institution: string; amountCents: number; asOfDate: string; interestRateBasisPoints: number | null; minimumPaymentCents: number | null; cadence: string; expectedChangeDate: string; expectedChangeAmountCents: number | null; notes: string; isArchived: boolean; createdAt: string; modifiedAt: string };
export type RetirementFinancialRecordInput = Omit<RetirementFinancialRecord, "id" | "createdAt" | "modifiedAt" | "isArchived"> & { id?: string };

export const getProtectedStoreStatus = () => invoke<ProtectedStoreStatus>("get_retirement_protected_store_status");
export const initializeProtectedStore = () => invoke<ProtectedStoreStatus>("initialize_retirement_protected_store");
export const unlockProtectedStore = () => invoke<ProtectedStoreStatus>("unlock_retirement_protected_store");
export const lockProtectedStore = () => invoke<ProtectedStoreStatus>("lock_retirement_protected_store");
export const getRetirementProfile = () => invoke<RetirementProfile>("get_retirement_protected_profile");
export const saveRetirementProfile = (input: RetirementProfile) => invoke<RetirementProfile>("save_retirement_protected_profile", { input });
export const listFinancialRecords = (entityType: RetirementFinancialRecord["entityType"]) => invoke<RetirementFinancialRecord[]>("list_retirement_financial_records", { entityType });
export const saveFinancialRecord = (input: RetirementFinancialRecordInput) => invoke<RetirementFinancialRecord>("save_retirement_financial_record", { input });
export const archiveFinancialRecord = (id: string, entityType: RetirementFinancialRecord["entityType"]) => invoke<void>("archive_retirement_financial_record", { id, entityType });
