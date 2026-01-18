"""ASCII game state renderer using rich.

Provides visual representation of game states for debugging,
tutorials, and understanding agent behavior.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING

import numpy as np

if TYPE_CHECKING:
    from essence_wars._core import PyGame

# Try to import rich, fall back to simple print if not available
try:
    from rich import box
    from rich.console import Console
    from rich.panel import Panel
    from rich.table import Table
    from rich.text import Text
    RICH_AVAILABLE = True
except ImportError:
    RICH_AVAILABLE = False


# Tensor layout constants (must match src/tensor.rs)
GLOBAL_SIZE = 6
PLAYER_STATE_SIZE = 75
BASE_STATS_SIZE = 5
HAND_CARDS_SIZE = 10
CREATURE_SLOTS = 5
CREATURE_SLOT_SIZE = 10
SUPPORT_SLOTS = 2
SUPPORT_SLOT_SIZE = 5

# Keyword bit values
KEYWORDS = {
    0x0001: "Guard",
    0x0002: "Lethal",
    0x0004: "Lifesteal",
    0x0008: "Rush",
    0x0010: "Ranged",
    0x0020: "Piercing",
    0x0040: "Shield",
    0x0080: "Quick",
    0x0100: "Ephemeral",
    0x0200: "Regenerate",
    0x0400: "Stealth",
    0x0800: "Charge",
    0x1000: "Frenzy",
    0x2000: "Volatile",
}


@dataclass
class CreatureInfo:
    """Decoded creature information."""
    occupied: bool
    attack: int
    health: int
    max_health: int
    can_attack: bool
    exhausted: bool
    silenced: bool
    has_rush: bool
    has_guard: bool
    keywords_bits: int

    @property
    def keywords(self) -> list[str]:
        """Get list of keyword names."""
        kws = []
        for bit, name in KEYWORDS.items():
            if self.keywords_bits & bit:
                kws.append(name)
        return kws

    def short_keywords(self) -> str:
        """Get short keyword string."""
        abbrevs = []
        if self.has_guard:
            abbrevs.append("G")
        if self.has_rush:
            abbrevs.append("R")
        if self.keywords_bits & 0x0002:  # Lethal
            abbrevs.append("L")
        if self.keywords_bits & 0x0004:  # Lifesteal
            abbrevs.append("LS")
        if self.keywords_bits & 0x0010:  # Ranged
            abbrevs.append("Rn")
        if self.keywords_bits & 0x0020:  # Piercing
            abbrevs.append("P")
        if self.keywords_bits & 0x0040:  # Shield
            abbrevs.append("S")
        if self.keywords_bits & 0x0400:  # Stealth
            abbrevs.append("St")
        return ",".join(abbrevs) if abbrevs else ""


@dataclass
class SupportInfo:
    """Decoded support information."""
    occupied: bool
    durability: int
    card_id: int


@dataclass
class PlayerInfo:
    """Decoded player state."""
    life: int
    essence: int
    action_points: int
    deck_size: int
    hand_size: int
    hand_card_ids: list[int]
    creatures: list[CreatureInfo]
    supports: list[SupportInfo]


@dataclass
class GameStateInfo:
    """Decoded game state."""
    turn: int
    current_player: int
    game_over: bool
    winner: int  # -1 = no winner, 0 = P1, 1 = P2
    players: list[PlayerInfo]


def decode_tensor(tensor: np.ndarray) -> GameStateInfo:
    """Decode a state tensor into human-readable GameStateInfo."""
    idx = 0

    # Global state
    turn = int(tensor[idx] * 30)  # Denormalize
    idx += 1
    current_player = int(tensor[idx])
    idx += 1
    game_over = tensor[idx] > 0.5
    idx += 1
    winner = int(tensor[idx])
    idx += 1
    idx += 2  # Skip reserved

    # Player states
    players = []
    for _ in range(2):
        # Base stats
        life = int(tensor[idx] * 20)  # STARTING_LIFE = 20
        idx += 1
        essence = int(tensor[idx] * 10)  # MAX_ESSENCE = 10
        idx += 1
        action_points = int(tensor[idx] * 3)  # AP_PER_TURN = 3
        idx += 1
        deck_size = int(tensor[idx] * 30)  # MAX_DECK_SIZE = 30
        idx += 1
        hand_size = int(tensor[idx] * 10)  # MAX_HAND_SIZE = 10
        idx += 1

        # Hand card IDs
        hand_card_ids = []
        for _ in range(HAND_CARDS_SIZE):
            card_id = int(tensor[idx])
            if card_id > 0:
                hand_card_ids.append(card_id)
            idx += 1

        # Creatures
        creatures = []
        for _ in range(CREATURE_SLOTS):
            occupied = tensor[idx] > 0.5
            idx += 1
            attack = int(tensor[idx] * 10)
            idx += 1
            health = int(tensor[idx] * 10)
            idx += 1
            max_health = int(tensor[idx] * 10)
            idx += 1
            can_attack = tensor[idx] > 0.5
            idx += 1
            exhausted = tensor[idx] > 0.5
            idx += 1
            silenced = tensor[idx] > 0.5
            idx += 1
            has_rush = tensor[idx] > 0.5
            idx += 1
            has_guard = tensor[idx] > 0.5
            idx += 1
            keywords_bits = int(tensor[idx] * 65535)
            idx += 1

            creatures.append(CreatureInfo(
                occupied=occupied,
                attack=attack,
                health=health,
                max_health=max_health,
                can_attack=can_attack,
                exhausted=exhausted,
                silenced=silenced,
                has_rush=has_rush,
                has_guard=has_guard,
                keywords_bits=keywords_bits,
            ))

        # Supports
        supports = []
        for _ in range(SUPPORT_SLOTS):
            occupied = tensor[idx] > 0.5
            idx += 1
            durability = int(tensor[idx] * 5)
            idx += 1
            card_id = int(tensor[idx])
            idx += 1
            idx += 2  # Skip reserved

            supports.append(SupportInfo(
                occupied=occupied,
                durability=durability,
                card_id=card_id,
            ))

        players.append(PlayerInfo(
            life=life,
            essence=essence,
            action_points=action_points,
            deck_size=deck_size,
            hand_size=hand_size,
            hand_card_ids=hand_card_ids,
            creatures=creatures,
            supports=supports,
        ))

    return GameStateInfo(
        turn=turn,
        current_player=current_player,
        game_over=game_over,
        winner=winner,
        players=players,
    )


class GameRenderer:
    """Renders game state as ASCII art using rich."""

    def __init__(self, use_color: bool = True):
        """Initialize renderer.

        Args:
            use_color: Whether to use colored output (requires rich)
        """
        self.use_color = use_color and RICH_AVAILABLE
        if self.use_color:
            self.console = Console()

    def render(self, game: PyGame) -> str:
        """Render the current game state.

        Args:
            game: PyGame instance to render

        Returns:
            String representation of game state
        """
        tensor = np.array(game.observe())
        mask = np.array(game.action_mask())
        state = decode_tensor(tensor)

        if self.use_color:
            return self._render_rich(state, mask, game)
        else:
            return self._render_plain(state, mask)

    def _render_rich(self, state: GameStateInfo, mask: np.ndarray, game: PyGame) -> str:
        """Render with rich formatting."""
        from io import StringIO

        output = StringIO()
        console = Console(file=output, force_terminal=True, width=80)

        # Header
        status = "GAME OVER" if state.game_over else f"Turn {state.turn}"
        if state.game_over:
            if state.winner == 0:
                status += " - Player 1 Wins!"
            elif state.winner == 1:
                status += " - Player 2 Wins!"
            else:
                status += " - Draw!"

        header = Text(f"=== Essence Wars: {status} ===", style="bold cyan")
        console.print(header, justify="center")
        console.print()

        # Player 2 (top, opponent perspective)
        self._render_player_rich(console, state.players[1], 2, state.current_player == 1)

        # Divider
        console.print("─" * 60, style="dim", justify="center")

        # Player 1 (bottom, our perspective)
        self._render_player_rich(console, state.players[0], 1, state.current_player == 0)

        # Legal actions summary
        num_legal = int(mask.sum())
        console.print()
        console.print(f"Legal actions: {num_legal}", style="dim")

        return output.getvalue()

    def _render_player_rich(
        self,
        console: Console,
        player: PlayerInfo,
        player_num: int,
        is_active: bool
    ) -> None:
        """Render a single player's state."""
        # Player header
        active_marker = " [*]" if is_active else ""
        style = "bold green" if is_active else "bold white"
        console.print(f"Player {player_num}{active_marker}", style=style)

        # Stats line
        stats = f"  Life: {player.life:2d}  |  Essence: {player.essence:2d}  |  AP: {player.action_points}  |  Hand: {player.hand_size}  |  Deck: {player.deck_size}"
        console.print(stats)

        # Board - Creatures
        creature_strs = []
        for i, c in enumerate(player.creatures):
            if c.occupied:
                status = ""
                if c.exhausted:
                    status = "z"
                elif c.can_attack:
                    status = "!"
                kw = c.short_keywords()
                kw_str = f" [{kw}]" if kw else ""
                creature_strs.append(f"[{c.attack}/{c.health}]{status}{kw_str}")
            else:
                creature_strs.append("[    ]")

        console.print("  Board: " + "  ".join(creature_strs))

        # Supports
        support_strs = []
        for s in player.supports:
            if s.occupied:
                support_strs.append(f"[S:{s.durability}]")
            else:
                support_strs.append("[   ]")
        if any(s.occupied for s in player.supports):
            console.print("  Supports: " + "  ".join(support_strs))

        console.print()

    def _render_plain(self, state: GameStateInfo, mask: np.ndarray) -> str:
        """Render without rich (fallback)."""
        lines = []

        # Header
        status = "GAME OVER" if state.game_over else f"Turn {state.turn}"
        lines.append(f"=== Essence Wars: {status} ===")
        lines.append("")

        # Players
        for i, player in enumerate(state.players):
            player_num = i + 1
            active = "*" if state.current_player == i else " "
            lines.append(f"Player {player_num} {active}")
            lines.append(f"  Life: {player.life}  Essence: {player.essence}  AP: {player.action_points}  Hand: {player.hand_size}  Deck: {player.deck_size}")

            # Creatures
            creature_strs = []
            for c in player.creatures:
                if c.occupied:
                    creature_strs.append(f"[{c.attack}/{c.health}]")
                else:
                    creature_strs.append("[    ]")
            lines.append("  Board: " + "  ".join(creature_strs))
            lines.append("")

        # Legal actions
        num_legal = int(mask.sum())
        lines.append(f"Legal actions: {num_legal}")

        return "\n".join(lines)

    def print(self, game: PyGame) -> None:
        """Print the rendered game state to console."""
        if self.use_color:
            tensor = np.array(game.observe())
            mask = np.array(game.action_mask())
            state = decode_tensor(tensor)

            # Print directly with rich
            self._print_rich(state, mask, game)
        else:
            print(self.render(game))

    def _print_rich(self, state: GameStateInfo, mask: np.ndarray, game: PyGame) -> None:
        """Print directly with rich console."""
        console = self.console

        # Clear and print header
        status = "GAME OVER" if state.game_over else f"Turn {state.turn}"
        if state.game_over:
            if state.winner == 0:
                status += " - Player 1 Wins!"
            elif state.winner == 1:
                status += " - Player 2 Wins!"
            else:
                status += " - Draw!"

        console.print()
        console.print(f"[bold cyan]=== Essence Wars: {status} ===[/]", justify="center")
        console.print()

        # Player 2 (opponent)
        self._render_player_rich(console, state.players[1], 2, state.current_player == 1)

        # Divider
        console.print("[dim]" + "─" * 60 + "[/]", justify="center")

        # Player 1 (us)
        self._render_player_rich(console, state.players[0], 1, state.current_player == 0)

        # Legal actions
        num_legal = int(mask.sum())
        console.print(f"[dim]Legal actions: {num_legal}[/]")


def render_game_state(game: PyGame, use_color: bool = True) -> str:
    """Convenience function to render a game state.

    Args:
        game: PyGame instance
        use_color: Whether to use colored output

    Returns:
        Rendered string
    """
    renderer = GameRenderer(use_color=use_color)
    return renderer.render(game)


def print_game_state(game: PyGame) -> None:
    """Convenience function to print a game state.

    Args:
        game: PyGame instance
    """
    renderer = GameRenderer()
    renderer.print(game)
