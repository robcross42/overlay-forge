import { invoke } from "@tauri-apps/api/core";

export type RepairResellSource = {
  id: string;
  kindKey: string;
  kindLabel: string;
  sourceKey: string;
  name: string;
  baseUrl: string;
  regionLabel: string;
  scrapeMode: "public_http" | "manual_import" | "disabled";
  adapterKey: string;
  enabled: boolean;
  priority: number;
  rateLimitSeconds: number;
  notes: string;
  lastScrapedAt: string;
  createdAt: string;
  modifiedAt: string;
};

export type RepairResellCategory = {
  id: string;
  key: string;
  label: string;
};

export type RepairResellKeywordFlag = {
  id: string;
  key: string;
  label: string;
  flagType: "opportunity" | "risk" | "info";
  pattern: string;
};

export type RepairResellListing = {
  id: string;
  sourceId: string;
  sourceName: string;
  sourceKey: string;
  externalId: string;
  canonicalUrl: string;
  title: string;
  normalizedTitle: string;
  descriptionText: string;
  sourceCategoryText: string;
  make: string;
  model: string;
  modelYear: number | null;
  lotNumber: string;
  conditionText: string;
  locationText: string;
  currencyCode: string;
  currentPriceCents: number | null;
  bidCount: number | null;
  closingAt: string;
  postedAt: string;
  lastSeenAt: string;
  listingStatus: "active" | "closed" | "sold" | "removed" | "unknown";
  pickupText: string;
  inspectionText: string;
  isWatchlisted: boolean;
  createdAt: string;
  modifiedAt: string;
  flags: RepairResellKeywordFlag[];
  categories: RepairResellCategory[];
};

export type RepairResellTravelProfile = {
  id: string;
  name: string;
  homeLocationLabel: string;
  vehicleLabel: string;
  fuelLPer100Km: number | null;
  fuelPriceCentsPerLitre: number | null;
  defaultRoundTripKm: number | null;
  notes: string;
  isDefault: boolean;
};

export type RepairResellDealEstimate = {
  id: string;
  listingId: string;
  travelProfileId: string;
  estimateLabel: string;
  acquisitionPriceCents: number | null;
  buyerPremiumCents: number | null;
  taxCents: number | null;
  travelKm: number | null;
  fuelCostCents: number | null;
  partsCostCents: number | null;
  otherCostCents: number | null;
  expectedResaleLowCents: number | null;
  expectedResaleHighCents: number | null;
  expectedResaleTargetCents: number | null;
  desiredProfitCents: number | null;
  riskBufferCents: number | null;
  maxSafeBidCents: number | null;
  netProfitLowCents: number | null;
  netProfitTargetCents: number | null;
  estimateMethod: "manual" | "rule" | "llm";
  confidence: "low" | "medium" | "high";
  notes: string;
  createdAt: string;
  modifiedAt: string;
};

export type RepairResellManualImportInput = {
  sourceId: string;
  canonicalUrl: string;
  title: string;
  descriptionText: string;
  sourceCategoryText?: string;
  conditionText?: string;
  locationText?: string;
  currentPriceCents?: number | null;
  closingAt?: string;
  pickupText?: string;
  inspectionText?: string;
};

export type RepairResellDealEstimateInput = {
  listingId: string;
  travelProfileId?: string;
  estimateLabel: string;
  acquisitionPriceCents?: number | null;
  buyerPremiumCents?: number | null;
  taxCents?: number | null;
  travelKm?: number | null;
  fuelCostCents?: number | null;
  partsCostCents?: number | null;
  otherCostCents?: number | null;
  expectedResaleLowCents?: number | null;
  expectedResaleHighCents?: number | null;
  expectedResaleTargetCents?: number | null;
  desiredProfitCents?: number | null;
  riskBufferCents?: number | null;
  confidence?: "low" | "medium" | "high";
  notes?: string;
};

export type RepairResellRefreshResult = {
  source: RepairResellSource;
  run: {
    id: string;
    status: string;
    listingCount: number;
    errorText: string;
  };
  listings: RepairResellListing[];
};

export function listRepairResellSources() {
  return invoke<RepairResellSource[]>("list_repair_resell_sources");
}

export function updateRepairResellSourceEnabled(sourceId: string, enabled: boolean) {
  return invoke<RepairResellSource>("update_repair_resell_source_enabled", { sourceId, enabled });
}

export function listRepairResellCategories() {
  return invoke<RepairResellCategory[]>("list_repair_resell_categories");
}

export function listRepairResellKeywordFlags() {
  return invoke<RepairResellKeywordFlag[]>("list_repair_resell_keyword_flags");
}

export function listRepairResellTravelProfiles() {
  return invoke<RepairResellTravelProfile[]>("list_repair_resell_travel_profiles");
}

export function listRepairResellListings() {
  return invoke<RepairResellListing[]>("list_repair_resell_listings");
}

export function manualImportRepairResellListing(input: RepairResellManualImportInput) {
  return invoke<RepairResellListing>("manual_import_repair_resell_listing", { input });
}

export function refreshRepairResellSource(sourceId: string) {
  return invoke<RepairResellRefreshResult>("refresh_repair_resell_source", { sourceId });
}

export function setRepairResellListingWatchlist(
  listingId: string,
  isWatchlisted: boolean,
  watchStatus = "watching",
  notes = ""
) {
  return invoke<RepairResellListing>("set_repair_resell_listing_watchlist", {
    input: { listingId, isWatchlisted, watchStatus, notes }
  });
}

export function listRepairResellDealEstimates(listingId: string) {
  return invoke<RepairResellDealEstimate[]>("list_repair_resell_deal_estimates", { listingId });
}

export function saveRepairResellDealEstimate(input: RepairResellDealEstimateInput) {
  return invoke<RepairResellDealEstimate>("save_repair_resell_deal_estimate", { input });
}
