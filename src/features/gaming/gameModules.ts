export type SupportedGameModuleView =
  | "builds"
  | "skill-tree"
  | "items"
  | "skill-gems"
  | "support-gems"
  | "loot-filter"
  | "trade"
  | "wizards"
  | "spells"
  | "upgrades"
  | "synergies"
  | "runs"
  | "vehicle-builds"
  | "building-reference"
  | "modding";

export type SupportedGameModuleSection = {
  view: SupportedGameModuleView;
  label: string;
  eyebrow: string;
  description: string;
};

export type SupportedGameModule = {
  slug: string;
  name: string;
  eyebrow: string;
  sections: SupportedGameModuleSection[];
  showScreenshots: boolean;
};

export const PATH_OF_EXILE_2_SLUG = "path-of-exile-2";
export const THE_SPELL_BRIGADE_SLUG = "the-spell-brigade";
export const TRAILMAKERS_SLUG = "trailmakers";

const SUPPORTED_GAME_MODULES: SupportedGameModule[] = [
  {
    slug: PATH_OF_EXILE_2_SLUG,
    name: "Path of Exile 2",
    eyebrow: "Module Scaffold",
    showScreenshots: false,
    sections: [
      {
        view: "builds",
        label: "Builds",
        eyebrow: "Character Planning",
        description:
          "Planned location for character builds, ascendancy choices, campaign notes, and endgame goals."
      },
      {
        view: "skill-tree",
        label: "Skill Tree",
        eyebrow: "Passive Planning",
        description:
          "Planned location for passive tree routes, keystones, weapon swaps, and respec notes."
      },
      {
        view: "items",
        label: "Items",
        eyebrow: "Equipment",
        description:
          "Planned location for gear targets, rare item notes, uniques, affixes, and upgrade priorities."
      },
      {
        view: "skill-gems",
        label: "Skill Gems",
        eyebrow: "Active Skills",
        description:
          "Planned location for active skill gems, gem levels, quality notes, and socket groups."
      },
      {
        view: "support-gems",
        label: "Support Gems",
        eyebrow: "Links",
        description:
          "Planned location for support choices, compatibility notes, and damage or utility links."
      },
      {
        view: "loot-filter",
        label: "Loot Filter",
        eyebrow: "Drops",
        description:
          "Planned location for local loot-filter rules, highlighting priorities, and filter exports."
      },
      {
        view: "trade",
        label: "Trade",
        eyebrow: "Market",
        description:
          "Planned location for saved trade lookups, upgrade searches, and price-check notes."
      }
    ]
  },
  {
    slug: THE_SPELL_BRIGADE_SLUG,
    name: "The Spell Brigade",
    eyebrow: "Co-op Survivors Planning",
    showScreenshots: true,
    sections: [
      {
        view: "wizards",
        label: "Wizards",
        eyebrow: "Characters",
        description:
          "Planned location for wizard choices, starter spells, roles, and team composition notes."
      },
      {
        view: "spells",
        label: "Spells",
        eyebrow: "Arsenal",
        description:
          "Planned location for spell notes, elements, behavior, and preferred upgrade paths."
      },
      {
        view: "upgrades",
        label: "Upgrades",
        eyebrow: "Progression",
        description:
          "Planned location for character and spell upgrades, unlock priorities, and progression notes."
      },
      {
        view: "synergies",
        label: "Synergies",
        eyebrow: "Team Builds",
        description:
          "Planned location for spell combinations, co-op roles, and team-wide build ideas."
      },
      {
        view: "runs",
        label: "Runs",
        eyebrow: "Objectives",
        description:
          "Planned location for team objectives, run outcomes, successful combinations, and lessons learned."
      }
    ]
  },
  {
    slug: TRAILMAKERS_SLUG,
    name: "Trailmakers",
    eyebrow: "Vehicle Building Workspace",
    showScreenshots: true,
    sections: [
      {
        view: "vehicle-builds",
        label: "Vehicle Builds",
        eyebrow: "Build Planning",
        description:
          "Local workspace for vehicle goals, block choices, controls, logic, tuning, iteration notes, and build-specific chat."
      },
      {
        view: "building-reference",
        label: "Build Reference",
        eyebrow: "Gameplay Authority",
        description:
          "Gameplay research anchored to the requested Trailmakers Wiki reference; version-sensitive guidance must be checked against the live source or identified as unverified."
      },
      {
        view: "modding",
        label: "Modding",
        eyebrow: "Lua Workspace",
        description:
          "Future Lua mod work grounded in the official wiki.gg modding guide and the API definitions and examples shipped in the installed mods directory."
      }
    ]
  }
];

export function getSupportedGameModule(slug: string) {
  return SUPPORTED_GAME_MODULES.find((module) => module.slug === slug) ?? null;
}

export function isSupportedGameModuleView(view: string): view is SupportedGameModuleView {
  return SUPPORTED_GAME_MODULES.some((module) =>
    module.sections.some((section) => section.view === view)
  );
}
