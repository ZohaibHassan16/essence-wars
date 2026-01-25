// Configuration
const CARDS_PATH = '/crates/essence-wars-ui/static/cards/core_set';
const CARD_DATA_PATH = '/data/cards/core_set';

// State
let allCards = [];
let markedCards = new Set();
let currentFilter = { faction: 'all', status: 'all' };
let searchQuery = '';

// Load card data from YAML files
async function loadCardData() {
    const factions = ['argentum', 'symbiote', 'obsidion', 'neutral'];
    const cardMap = new Map();

    for (const faction of factions) {
        try {
            const response = await fetch(`${CARD_DATA_PATH}/${faction}.yaml`);
            const yamlText = await response.text();
            
            // Parse YAML manually (simple parser for this structure)
            const cards = parseYAML(yamlText, faction);
            cards.forEach(card => cardMap.set(card.id, card));
        } catch (error) {
            console.error(`Error loading ${faction} cards:`, error);
        }
    }

    return cardMap;
}

// Simple YAML parser for card data
function parseYAML(yamlText, faction) {
    const cards = [];
    const lines = yamlText.split('\n');
    let currentCard = null;
    let inCards = false;

    for (let i = 0; i < lines.length; i++) {
        const line = lines[i];
        
        if (line.includes('cards:')) {
            inCards = true;
            continue;
        }

        if (!inCards) continue;

        // New card entry
        if (line.match(/^\s*- id:\s*(\d+)/)) {
            if (currentCard) {
                cards.push(currentCard);
            }
            const id = parseInt(line.match(/- id:\s*(\d+)/)[1]);
            currentCard = { id, faction, name: 'Unknown' };
        }
        
        // Card name
        if (currentCard && line.match(/^\s+name:\s*"(.+)"/)) {
            currentCard.name = line.match(/name:\s*"(.+)"/)[1];
        }
    }

    if (currentCard) {
        cards.push(currentCard);
    }

    return cards;
}

// Load images from directory
async function loadImages() {
    // Since we can't directly list directory contents from browser,
    // we'll infer from card IDs
    const ranges = [
        { start: 1000, end: 1074, faction: 'argentum' },
        { start: 2000, end: 2074, faction: 'symbiote' },
        { start: 3000, end: 3074, faction: 'obsidion' },
        { start: 4000, end: 4074, faction: 'neutral' }
    ];

    const images = [];
    for (const range of ranges) {
        for (let id = range.start; id <= range.end; id++) {
            images.push({
                id,
                path: `${CARDS_PATH}/${id}.webp`,
                faction: range.faction
            });
        }
    }

    return images;
}

// Initialize the app
async function init() {
    console.log('Loading card data...');
    const cardMap = await loadCardData();
    const images = await loadImages();

    // Combine image and card data
    allCards = images.map(img => ({
        ...img,
        name: cardMap.get(img.id)?.name || `Card ${img.id}`,
        exists: true // We'll check this when rendering
    }));

    // Load saved marks from localStorage
    const saved = localStorage.getItem('markedCards');
    if (saved) {
        markedCards = new Set(JSON.parse(saved));
    }

    console.log(`Loaded ${allCards.length} cards`);
    renderCards();
    updateStats();
    setupEventListeners();
}

// Render cards
function renderCards() {
    const grid = document.getElementById('grid');
    const filteredCards = getFilteredCards();

    if (filteredCards.length === 0) {
        grid.innerHTML = '<div class="loading">No cards match your filters</div>';
        return;
    }

    grid.innerHTML = filteredCards.map(card => `
        <div class="card ${markedCards.has(card.id) ? 'marked' : ''}" data-id="${card.id}">
            <div class="card-image-container">
                <div class="card-title">${card.name}</div>
                <img class="card-image" 
                     src="${card.path}" 
                     alt="${card.name}"
                     onerror="this.src='data:image/svg+xml,%3Csvg xmlns=%22http://www.w3.org/2000/svg%22 width=%22280%22 height=%22392%22%3E%3Crect fill=%22%23333%22 width=%22280%22 height=%22392%22/%3E%3Ctext x=%2250%25%22 y=%2250%25%22 text-anchor=%22middle%22 fill=%22%23fff%22 font-size=%2220%22%3EImage Missing%3C/text%3E%3C/svg%3E'">
                <div class="card-id">#${card.id}</div>
            </div>
            <div class="card-info">
                <span class="card-faction faction-${card.faction}">${card.faction}</span>
                <button class="mark-btn ${markedCards.has(card.id) ? 'marked' : 'unmarked'}" 
                        onclick="toggleMark(${card.id}, event)">
                    ${markedCards.has(card.id) ? '✓ Marked for Regeneration' : 'Mark for Regeneration'}
                </button>
            </div>
        </div>
    `).join('');
}

// Filter cards
function getFilteredCards() {
    return allCards.filter(card => {
        // Faction filter
        if (currentFilter.faction !== 'all' && card.faction !== currentFilter.faction) {
            return false;
        }

        // Status filter
        if (currentFilter.status === 'marked' && !markedCards.has(card.id)) {
            return false;
        }
        if (currentFilter.status === 'unmarked' && markedCards.has(card.id)) {
            return false;
        }

        // Search filter
        if (searchQuery) {
            const query = searchQuery.toLowerCase();
            return card.name.toLowerCase().includes(query) || 
                   card.id.toString().includes(query);
        }

        return true;
    });
}

// Toggle mark
function toggleMark(cardId, event) {
    event.stopPropagation();
    
    if (markedCards.has(cardId)) {
        markedCards.delete(cardId);
    } else {
        markedCards.add(cardId);
    }

    // Save to localStorage
    localStorage.setItem('markedCards', JSON.stringify([...markedCards]));

    // Update UI
    renderCards();
    updateStats();
}

// Update statistics
function updateStats() {
    document.getElementById('total-count').textContent = allCards.length;
    document.getElementById('marked-count').textContent = markedCards.size;
}

// Export marked cards
function exportMarkedCards() {
    if (markedCards.size === 0) {
        alert('No cards marked for regeneration!');
        return;
    }

    const markedCardsData = allCards
        .filter(card => markedCards.has(card.id))
        .map(card => ({
            id: card.id,
            name: card.name,
            faction: card.faction,
            file_path: card.path,
            absolute_path: `/home/chris/ai-cardgame/crates/essence-wars-ui/static/cards/core_set/${card.id}.webp`
        }));

    const exportData = {
        exported_at: new Date().toISOString(),
        total_marked: markedCards.size,
        cards: markedCardsData
    };

    // Create and download JSON file
    const blob = new Blob([JSON.stringify(exportData, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `marked-cards-${Date.now()}.json`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);

    // Show modal
    showModal(exportData);
}

// Show export modal
function showModal(data) {
    const modal = document.getElementById('modal');
    const preview = document.getElementById('export-preview');
    preview.textContent = JSON.stringify(data, null, 2);
    modal.classList.add('active');
}

// Close modal
function closeModal() {
    document.getElementById('modal').classList.remove('active');
}

// Clear all marks
function clearAllMarks() {
    if (markedCards.size === 0) {
        return;
    }

    if (confirm(`Clear all ${markedCards.size} marked cards?`)) {
        markedCards.clear();
        localStorage.removeItem('markedCards');
        renderCards();
        updateStats();
    }
}

// Setup event listeners
function setupEventListeners() {
    // Search
    document.getElementById('search').addEventListener('input', (e) => {
        searchQuery = e.target.value;
        renderCards();
    });

    // Faction filters
    document.querySelectorAll('[data-faction]').forEach(btn => {
        btn.addEventListener('click', () => {
            document.querySelectorAll('[data-faction]').forEach(b => b.classList.remove('active'));
            btn.classList.add('active');
            currentFilter.faction = btn.dataset.faction;
            renderCards();
        });
    });

    // Status filters
    document.querySelectorAll('[data-filter]').forEach(btn => {
        btn.addEventListener('click', () => {
            // Toggle status filter
            if (btn.classList.contains('active')) {
                btn.classList.remove('active');
                currentFilter.status = 'all';
            } else {
                document.querySelectorAll('[data-filter]').forEach(b => b.classList.remove('active'));
                btn.classList.add('active');
                currentFilter.status = btn.dataset.filter;
            }
            renderCards();
        });
    });

    // Export button
    document.getElementById('export-btn').addEventListener('click', exportMarkedCards);

    // Clear button
    document.getElementById('clear-btn').addEventListener('click', clearAllMarks);

    // Card click to toggle
    document.getElementById('grid').addEventListener('click', (e) => {
        const card = e.target.closest('.card');
        if (card && !e.target.classList.contains('mark-btn')) {
            const cardId = parseInt(card.dataset.id);
            toggleMark(cardId, e);
        }
    });

    // Close modal on background click
    document.getElementById('modal').addEventListener('click', (e) => {
        if (e.target.id === 'modal') {
            closeModal();
        }
    });
}

// Start the app
init();
