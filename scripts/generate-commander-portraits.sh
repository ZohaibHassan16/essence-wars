#!/bin/bash
# =============================================================================
# ESSENCE WARS - Commander Portrait Generator (Premium Quality)
# =============================================================================
# Uses FLUX Dev model for high-quality portrait generation
# Output: static/portrait/{id}.webp
# =============================================================================

set -e

# Configuration
SD_DIR="$HOME/stable-diffusion.cpp"
MODEL_DIR="$HOME/.ai-assets/models/flux"
LORA_DIR="$HOME/.ai-assets/loras"
OUTPUT_DIR="$HOME/.ai-assets/output/portraits"
FINAL_DIR="/home/chris/ai-cardgame/crates/essence-wars-ui/static/portrait"

# Model files
DIFFUSION_MODEL="$MODEL_DIR/flux-dev-q8.gguf"
VAE="$MODEL_DIR/ae.safetensors"
CLIP_L="$MODEL_DIR/clip_l.safetensors"
T5XXL="$MODEL_DIR/t5-Q5_K_M.gguf"

# Generation settings (Dev = premium quality)
STEPS=20
CFG_SCALE=1.0
SAMPLING="euler"
WIDTH=704
HEIGHT=896

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  Essence Wars Commander Portraits${NC}"
echo -e "${BLUE}  Premium Quality (FLUX Dev)${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Create directories
mkdir -p "$OUTPUT_DIR"
mkdir -p "$FINAL_DIR"

# Function to generate a portrait
generate_portrait() {
    local id=$1
    local name=$2
    local prompt=$3
    local loras=$4

    # Convert name to snake_case for filename (e.g., "The High Artificer" -> "the_high_artificer")
    local filename=$(echo "$name" | tr '[:upper:]' '[:lower:]' | tr ' ' '_')

    echo -e "${YELLOW}[$id] Generating: $name -> ${filename}.webp${NC}"

    cd "$SD_DIR"

    ./build/bin/sd-cli \
        --diffusion-model "$DIFFUSION_MODEL" \
        --vae "$VAE" \
        --clip_l "$CLIP_L" \
        --t5xxl "$T5XXL" \
        --lora-model-dir "$LORA_DIR" \
        -p "$prompt, no text, no words, no letters $loras" \
        --cfg-scale $CFG_SCALE \
        --sampling-method $SAMPLING \
        --steps $STEPS \
        -W $WIDTH -H $HEIGHT \
        -o "$OUTPUT_DIR/${filename}.png"

    # Convert to WebP
    cwebp -q 95 "$OUTPUT_DIR/${filename}.png" -o "$FINAL_DIR/${filename}.webp"

    echo -e "${GREEN}[$id] Done: $FINAL_DIR/${filename}.webp${NC}"
    echo ""
}

# =============================================================================
# ARGENTUM COMBINE COMMANDERS
# =============================================================================

echo -e "${BLUE}--- Argentum Combine ---${NC}"

# 5000 - The High Artificer
generate_portrait 5000 "The High Artificer" \
"Three-quarter view waist-up portrait of a brilliant female artificer commander, showing from head to waist with inventor's poise. Ornate white and gold robes with brass mechanical augmentations visible on her shoulders, arms, and torso. Attractive confident face with intelligent piercing eyes gazing forward, brass goggles with intricate gears pushed up on her forehead. Multiple delicate mechanical arms from her brass backpack extending over her shoulders and around her body - one holding a glowing blueprint hologram, another assembling a tiny clockwork construct in mid-air near her hand. Her natural hands are positioned gracefully, one gesturing toward her creation, the other holding precision tools. Confident, innovative stance showing mastery of her craft. Warm golden light from forges creating soft rim lighting on her face, hair, and robes. Refined, elegant expression showing wisdom and authority. Workshop with floating blueprints and tool drones softly blurred in background. 90s Magic the Gathering card art, painted fantasy illustration with classical portrait composition" \
"<lora:rutkowski:0.4> <lora:classical-painting:0.7>"

# 5001 - The Sanctum Healer
generate_portrait 5001 "The Sanctum Healer" \
"Three-quarter view waist-up portrait of a graceful female healing artificer commander, showing from head to waist with serene authority. Elegant white robes with gold geometric Art Deco patterns and brass medical equipment visible on shoulders, arms, and torso. Beautiful compassionate face with warm intelligent eyes radiating wisdom and care, brass circlet with healing crystal on her forehead. Multiple delicate brass mechanical healing arms extending from an ornate backpack - one dispensing golden healing mist over a wounded soldier, another holding glowing essence vials, another adjusting medical instruments. Her natural hands positioned gracefully in a healing gesture, golden regeneration magic flowing from her palms. Calm, composed stance showing mastery over life and restoration. Warm golden light from healing essence creating soft ethereal glow around her entire form, like a medical saint. Hospital ward with floating medical holograms and healing chambers softly blurred in background. Serene, caring expression showing divine compassion and authority. 90s Magic the Gathering card art, painted fantasy illustration with refined classical lighting and soft healing aura" \
"<lora:rutkowski:0.4> <lora:classical-painting:0.7>"

# 5002 - Siege Marshal Vex
generate_portrait 5002 "Siege Marshal Vex" \
"Three-quarter view waist-up portrait of a battle-hardened male military commander, showing from head to waist with warrior's bearing. Heavy brass siege armor covering his torso with piercing weapon components integrated into shoulder guards, gauntlets, and chest plate. Steam venting from armor joints at neck, shoulders, and waist. Strong weathered face with a commanding scowl, battle scars across cheek and brow, intense eyes beneath furrowed brow radiating tactical authority. Short military-cut hair, grizzled stubble. One gauntleted hand gripping a heavy siege weapon at his side, the other clenched into a fist or pointing forward in command. Powerful military stance showing readiness for battle. War-torn battlefield with smoke and ember sparks softly out of focus in background. Dramatic side lighting from fires creating strong shadows on his face and armor, highlighting his determination and authority. Argentum war banner barely visible behind. 90s Magic the Gathering card art, gritty painted fantasy illustration with dynamic brushwork" \
"<lora:frazetta:0.4> <lora:classical-painting:0.6>"

# 5003 - The Grand Architect
generate_portrait 5003 "The Grand Architect" \
"Three-quarter view waist-up portrait of an elegant female architect commander, showing from head to waist in regal composition. Flowing white robes with gold geometric Art Deco patterns cascading over her shoulders and body, ornate brass circlet on her forehead with glowing crystal. Beautiful composed face with serene confidence, high cheekbones, gentle knowing smile radiating wisdom. Her hands positioned gracefully - one raised with golden magical energy flowing from her fingernails commanding floating holographic blueprints that orbit around her, the other hand holding an ornate brass compass and drafting tool. Multiple Art Deco framed blueprint holograms float in the air around her, glowing with golden light. Composed, visionary stance showing her mastery over design and construction. Divine soft golden light streaming from above, creating an almost ethereal glow around her entire form. White marble fortress construction with floating blueprint holograms softly blurred in background. Wise, peaceful expression radiating authority and vision. 90s Magic the Gathering card art, classical painted portrait with refined Renaissance-style lighting" \
"<lora:rutkowski:0.4> <lora:classical-painting:0.7>"

# =============================================================================
# SYMBIOTE CIRCLES COMMANDERS
# =============================================================================

echo -e "${BLUE}--- Symbiote Circles ---${NC}"

# 5004 - The Broodmother
generate_portrait 5004 "The Broodmother" \
"Three-quarter view waist-up portrait of a fierce female druid queen commander, showing from head to waist with primal majesty. Living bio-armor of bioluminescent purple vines and bone plates covering her shoulders, torso, and waist - a complete organic battle suit that pulses with life. Powerful athletic features with wild untamed hair flowing with living tendrils that pulse with purple light, some tendrils extending down past her shoulders. One clawed hand raised in a summoning gesture, the other hand positioned at her waist with fingers splayed showing predatory claws. Chitinous armor plating visible on both forearms and torso. Deep green skin with organic circuit-like patterns spreading across her body. Intense predatory gaze showing both beauty and danger, tribal markings glowing on her cheekbones and neck. Commanding stance of a pack leader ready to call her swarm. Jungle background with glowing spore trees and smaller symbiote creatures softly out of focus, purple and green bioluminescence creating dramatic rim lighting on her bio-armor. 90s Magic the Gathering card art, painted fantasy illustration with vibrant organic textures" \
"<lora:frazetta:0.5> <lora:classical-painting:0.5>"

# 5005 - Plague Sovereign
generate_portrait 5005 "Plague Sovereign" \
"Three-quarter view waist-up portrait of a regal female plague queen commander, showing from head to waist with haunting elegance. Flowing robes of living decay and membrane fabric draped over her body, edges dissolving into clouds of deadly spores. Ornate belt of bone and fungal growth at her waist holding vials of concentrated plague essence. Hauntingly beautiful pale face with sickly elegant features, glowing green eyes that pierce through shadow with ancient malevolence. One delicate hand raised to chest level summoning clouds of volatile spores that swirl around her fingers, the other hand positioned at her waist with an elegant but threatening gesture. Expression is both alluring and terrifying, aristocratic beauty touched by death and decay. Regal stance of a sovereign who commands pestilence itself. Throne of bones and fungal growth with spreading decay visible softly blurred behind her. Sickly green and purple bioluminescent lighting creating an otherworldly glow around her entire form, spore particles floating in the air catching the eerie light. 90s Magic the Gathering card art, dark fantasy painted portrait with unsettling beauty" \
"<lora:frazetta:0.5> <lora:classical-painting:0.5>"

# 5006 - Alpha of the Hunt
generate_portrait 5006 "Alpha of the Hunt" \
"Three-quarter view waist-up portrait of a savage male pack alpha commander, showing from head to waist with feral intensity and predatory power. Predator grafts covering his entire muscular torso - bone spurs and chitin plates naturally fused to his powerful frame, creating natural organic armor. Feral yellow eyes gleaming with hunting instinct, intense predatory gaze locked forward ready to strike. Strong angular face with tribal scars and bioluminescent green markings pulsing across cheekbones, neck, and chest. Wild mane of dark hair with living tendrils that move independently like sensing appendages. Powerful muscular arms and torso showing raw primal strength, hands positioned in a crouched aggressive stance - claws extended, one hand near his chest, the other reaching forward as if about to pounce. Hunting posture of an apex predator leading his pack into frenzy. Tribal bone ornaments and symbiote grafts visible at his waist. Dark jungle clearing behind with glowing eyes of his pack barely visible in shadows. Dramatic lighting from bioluminescent sources creating dangerous atmosphere, green light pulsing across his body in tribal patterns. 90s Magic the Gathering card art, dynamic painted fantasy illustration with aggressive energy" \
"<lora:frazetta:0.5> <lora:classical-painting:0.5>"

# 5007 - The Eternal Grove
generate_portrait 5007 "The Eternal Grove" \
"Three-quarter view waist-up portrait of a massive sentient tree-creature commander, ancient bark-face and trunk down to the base filling the frame with timeless presence. Face formed naturally in ancient weathered bark with deep wise eyes that glow with green bioluminescent life, showing eons of growth and protection. Texture of centuries-old wood with moss, vines, and healing sap flowing through cracks like glowing veins across the entire trunk. Crown of living branches forming a natural halo behind the head, small leaves and bioluminescent flowers blooming. Powerful branch-arms extending from the trunk - gnarled and ancient, positioned in a protective gesture. The trunk widens toward the base where roots begin to emerge and spread. Expression is ancient, wise, and protective - a guardian spirit given physical form. Smaller symbiotic creatures, birds, and parasites nestled in bark crevices visible across the trunk and shoulders. Regenerating moss and new growth sprouting along the body showing eternal life. Sacred grove background with smaller creatures sheltering beneath softly out of focus, glowing spore lights creating ethereal atmosphere. Warm green and purple bioluminescent lighting creating a mystical aura around the entire form, healing essence radiating outward. 90s Magic the Gathering card art, fantasy nature portrait with spiritual gravitas" \
"<lora:rutkowski:0.4> <lora:classical-painting:0.6>"

# =============================================================================
# OBSIDION SYNDICATE COMMANDERS
# =============================================================================

echo -e "${BLUE}--- Obsidion Syndicate ---${NC}"

# 5008 - The Blood Sovereign
generate_portrait 5008 "The Blood Sovereign" \
"Three-quarter view waist-up portrait of an alluring vampire queen commander, showing from head to waist in regal gothic composition. Elegant crimson velvet dress with gothic corset bodice cinched at waist, ornate golden embroidery and blood-red gems adorning the entire garment. Pale porcelain skin with aristocratic features - high cheekbones, full lips curved in a subtle knowing smile showing a hint of fangs. Eyes burning with crimson power and ancient intelligence that has ruled for centuries. Glowing blue essence tubes connected to ornate brass gauntlets on both wrists and forearms, pulsing with stolen life force. Both hands raised gracefully - one near her chest with blood magic swirling around her fingers granting power to servants, the other extended in a commanding gesture of dominion. Expression is confident, seductive, and commanding - absolute authority and dangerous allure combined. Regal stance of a sovereign who rules through blood and fear. Gothic throne room with stained glass depicting blood rituals and her noble lineage softly blurred behind. Dramatic lighting from essence tubes and blood magic creating crimson and blue highlights on her pale features and dress. 90s Magic the Gathering card art, gothic fantasy portrait painting with romantic darkness" \
"<lora:classical-painting:0.5> <lora:frazetta:0.4>"

# 5009 - The Deathmaster
generate_portrait 5009 "The Deathmaster" \
"Three-quarter view waist-up portrait of a deadly female assassin guildmaster commander, showing from head to waist with lethal poise and coiled readiness. Elegant black leather armor with dark crimson accents covering her torso, form-fitting yet practical for swift movement. Multiple concealed blade sheaths visible at her shoulders, forearms, and waist, each dagger coated with a faint green venomous glow. Beautiful pale features with sharp cheekbones and cold calculating eyes that have witnessed countless deaths, thin cruel smile suggesting she already knows how you'll die. Dark hair pulled back tightly in a functional style, revealing elegant pointed ears suggesting elven heritage. One gloved hand raised near her face holding a perfectly balanced throwing knife between her fingers, the other hand resting on a sheathed blade at her hip. Her posture suggests supernatural speed - ready to strike before her target can react. Expression is confident and predatory, a master of death who ensures every poisoned blade finds its mark. Faint wisps of shadow curl around her form like loyal servants. Gothic assassin's guild hall with weapon racks and training dummies softly blurred in background. Dramatic side lighting creating sharp shadows on her face, neon blue essence light glinting off her many blades. 90s Magic the Gathering card art, dark gothic portrait with lethal elegance" \
"<lora:frazetta:0.5> <lora:classical-painting:0.5>"

# 5010 - The Shadow Weaver
generate_portrait 5010 "The Shadow Weaver" \
"Three-quarter view waist-up portrait of a mysterious female shadow-shifter commander, showing from head to waist as her form constantly shifts between solid and shadow. Elegant dark robes with hood partially drawn, flowing into living darkness at the edges, the fabric of her garment seeming to be woven from shadow itself. Beautiful pale face that flickers - one moment solid and defined with elegant aristocratic features and piercing eyes, the next moment dissolving into shadow with only glowing violet eyes remaining visible. Dark hair that moves like smoke, sometimes solid silk, sometimes wisps of living shadow. Both hands visible in summoning poses - one raised near her chest weaving shadow essence between her fingers, the other extended at waist level with fingers trailing darkness as multiple shadow clones begin to emerge and peel away from her body. Her torso phases between corporeal and incorporeal, showing the constant flux of her nature. Expression is enigmatic and otherworldly, impossible to fully read - both present and absent. Stance suggests she could disappear entirely at any moment, becoming one with the darkness. Gothic cathedral with independently moving shadows and shadow duplicates softly out of focus behind. Lighting is dramatic and unstable - she seems to both absorb and emit darkness, with only hints of blue and violet essence light defining her constantly shifting form. 90s Magic the Gathering card art, ethereal dark fantasy portrait with supernatural energy" \
"<lora:frazetta:0.5> <lora:classical-painting:0.5>"

# 5011 - Void Archon
generate_portrait 5011 "Void Archon" \
"Three-quarter view waist-up portrait of a powerful female time mage commander, showing from head to waist with arcane mastery and temporal authority. Elegant black and neon blue robes with an alluring cut, arcane symbols and clock motifs embroidered in glowing thread across the fabric. Beautiful pale features with sharp intelligent eyes that seem to see past, present, and future simultaneously, glowing with blue temporal energy. Dark hair flowing as if caught in a temporal wind, strands moving at different speeds. Both hands raised in spellcasting poses - one manipulating a hovering hourglass with sand flowing upward, the other with fingers spread as blue time-acceleration energy crackles between them. Multiple clocks and hourglasses floating around her form, some running forward, some backward, some frozen. Expression is confident and knowing, a master of time who has seen all outcomes. Stance suggests coiled readiness, able to strike with supernatural speed. Gothic tower laboratory with temporal devices and swirling time portals softly blurred in background. Dramatic lighting from temporal magic creating neon blue highlights and temporal distortion effects around her entire form. 90s Magic the Gathering card art, arcane fantasy portrait with supernatural temporal energy" \
"<lora:frazetta:0.5> <lora:classical-painting:0.5>"

# =============================================================================
# SUMMARY
# =============================================================================

echo ""
echo -e "${BLUE}========================================${NC}"
echo -e "${GREEN}  All 12 Commander Portraits Generated!${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""
echo "PNG files: $OUTPUT_DIR/"
echo "WebP files: $FINAL_DIR/"
echo ""
echo "Commanders generated:"
echo "  Argentum:  5000, 5001, 5002, 5003"
echo "  Symbiote:  5004, 5005, 5006, 5007"
echo "  Obsidion:  5008, 5009, 5010, 5011"
echo ""
