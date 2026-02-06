/**
 * IndexedDB storage implementation for web platform.
 *
 * Provides persistent storage for:
 * - Replays (SpectatorMatch objects)
 * - Custom decks
 * - Settings (future use)
 */

const DB_NAME = "essence-wars";
const DB_VERSION = 1;

// Store names
const STORES = {
  REPLAYS: "replays",
  CUSTOM_DECKS: "customDecks",
  SETTINGS: "settings",
} as const;

/**
 * Stored replay record
 */
export interface StoredReplay {
  id: string;
  name: string;
  timestamp: number;
  player1DeckName: string;
  player2DeckName: string;
  player1Type: string;
  player2Type: string;
  winner: 1 | 2 | null;
  totalTurns: number;
  totalActions: number;
  /** The full match data (serialized) */
  data: string;
}

/**
 * Stored custom deck record
 */
export interface StoredCustomDeck {
  id: string;
  name: string;
  commander: number;
  cards: number[];
  description: string;
  tags: string[];
  createdAt: string;
  modifiedAt: string;
}

let db: IDBDatabase | null = null;

/**
 * Open/create the IndexedDB database.
 */
export async function openDatabase(): Promise<IDBDatabase> {
  if (db) {
    return db;
  }

  return new Promise((resolve, reject) => {
    const request = indexedDB.open(DB_NAME, DB_VERSION);

    request.onerror = () => {
      console.error("[IndexedDB] Failed to open database:", request.error);
      reject(new Error(`Failed to open IndexedDB: ${request.error?.message}`));
    };

    request.onsuccess = () => {
      db = request.result;
      console.log("[IndexedDB] Database opened successfully");
      resolve(db);
    };

    request.onupgradeneeded = (event) => {
      console.log("[IndexedDB] Upgrading database schema...");
      const database = (event.target as IDBOpenDBRequest).result;

      // Create replays store
      if (!database.objectStoreNames.contains(STORES.REPLAYS)) {
        const replayStore = database.createObjectStore(STORES.REPLAYS, {
          keyPath: "id",
        });
        replayStore.createIndex("timestamp", "timestamp", { unique: false });
        replayStore.createIndex("name", "name", { unique: false });
        console.log("[IndexedDB] Created replays store");
      }

      // Create custom decks store
      if (!database.objectStoreNames.contains(STORES.CUSTOM_DECKS)) {
        const deckStore = database.createObjectStore(STORES.CUSTOM_DECKS, {
          keyPath: "id",
        });
        deckStore.createIndex("name", "name", { unique: false });
        deckStore.createIndex("modifiedAt", "modifiedAt", { unique: false });
        console.log("[IndexedDB] Created customDecks store");
      }

      // Create settings store
      if (!database.objectStoreNames.contains(STORES.SETTINGS)) {
        database.createObjectStore(STORES.SETTINGS, { keyPath: "key" });
        console.log("[IndexedDB] Created settings store");
      }
    };
  });
}

/**
 * Close the database connection.
 */
export function closeDatabase(): void {
  if (db) {
    db.close();
    db = null;
  }
}

// =============================================================================
// Generic helpers
// =============================================================================

async function getStore(
  storeName: string,
  mode: IDBTransactionMode = "readonly"
): Promise<IDBObjectStore> {
  const database = await openDatabase();
  const transaction = database.transaction(storeName, mode);
  return transaction.objectStore(storeName);
}

async function promisifyRequest<T>(request: IDBRequest<T>): Promise<T> {
  return new Promise((resolve, reject) => {
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error);
  });
}

// =============================================================================
// Replay Storage
// =============================================================================

/**
 * Save a replay to IndexedDB.
 */
export async function saveReplay(replay: StoredReplay): Promise<void> {
  const store = await getStore(STORES.REPLAYS, "readwrite");
  await promisifyRequest(store.put(replay));
  console.log(`[IndexedDB] Saved replay: ${replay.id}`);
}

/**
 * Get all replays, sorted by timestamp (newest first).
 */
export async function getAllReplays(): Promise<StoredReplay[]> {
  const store = await getStore(STORES.REPLAYS, "readonly");
  const replays = await promisifyRequest(store.getAll());
  // Sort by timestamp descending (newest first)
  return replays.sort((a, b) => b.timestamp - a.timestamp);
}

/**
 * Get a single replay by ID.
 */
export async function getReplay(id: string): Promise<StoredReplay | undefined> {
  const store = await getStore(STORES.REPLAYS, "readonly");
  return await promisifyRequest(store.get(id));
}

/**
 * Delete a replay by ID.
 */
export async function deleteReplay(id: string): Promise<void> {
  const store = await getStore(STORES.REPLAYS, "readwrite");
  await promisifyRequest(store.delete(id));
  console.log(`[IndexedDB] Deleted replay: ${id}`);
}

/**
 * Get the count of stored replays.
 */
export async function getReplayCount(): Promise<number> {
  const store = await getStore(STORES.REPLAYS, "readonly");
  return await promisifyRequest(store.count());
}

// =============================================================================
// Custom Deck Storage
// =============================================================================

/**
 * Save a custom deck to IndexedDB.
 */
export async function saveCustomDeck(deck: StoredCustomDeck): Promise<void> {
  const store = await getStore(STORES.CUSTOM_DECKS, "readwrite");
  await promisifyRequest(store.put(deck));
  console.log(`[IndexedDB] Saved custom deck: ${deck.id}`);
}

/**
 * Get all custom decks, sorted by modified date (newest first).
 */
export async function getAllCustomDecks(): Promise<StoredCustomDeck[]> {
  const store = await getStore(STORES.CUSTOM_DECKS, "readonly");
  const decks = await promisifyRequest(store.getAll());
  // Sort by modifiedAt descending (newest first)
  return decks.sort(
    (a, b) => new Date(b.modifiedAt).getTime() - new Date(a.modifiedAt).getTime()
  );
}

/**
 * Get a single custom deck by ID.
 */
export async function getCustomDeck(
  id: string
): Promise<StoredCustomDeck | undefined> {
  const store = await getStore(STORES.CUSTOM_DECKS, "readonly");
  return await promisifyRequest(store.get(id));
}

/**
 * Delete a custom deck by ID.
 */
export async function deleteCustomDeck(id: string): Promise<void> {
  const store = await getStore(STORES.CUSTOM_DECKS, "readwrite");
  await promisifyRequest(store.delete(id));
  console.log(`[IndexedDB] Deleted custom deck: ${id}`);
}

/**
 * Get the count of stored custom decks.
 */
export async function getCustomDeckCount(): Promise<number> {
  const store = await getStore(STORES.CUSTOM_DECKS, "readonly");
  return await promisifyRequest(store.count());
}

// =============================================================================
// Settings Storage (for future use)
// =============================================================================

/**
 * Save a setting.
 */
export async function saveSetting(key: string, value: unknown): Promise<void> {
  const store = await getStore(STORES.SETTINGS, "readwrite");
  await promisifyRequest(store.put({ key, value }));
}

/**
 * Get a setting.
 */
export async function getSetting<T>(key: string): Promise<T | undefined> {
  const store = await getStore(STORES.SETTINGS, "readonly");
  const result = await promisifyRequest(store.get(key));
  return result?.value as T | undefined;
}

/**
 * Delete a setting.
 */
export async function deleteSetting(key: string): Promise<void> {
  const store = await getStore(STORES.SETTINGS, "readwrite");
  await promisifyRequest(store.delete(key));
}
