/**
 * Match Statistics Module
 *
 * Provides comprehensive statistics computation and export for Essence Wars matches.
 */

// Types
export * from "./types";

// Computation
export { computeMatchStatistics } from "./statsComputer";

// Export utilities
export {
  exportToJson,
  exportToCsv,
  downloadJson,
  downloadCsv,
  downloadFile,
} from "./export";
