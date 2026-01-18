"""Main Benchmark API for Essence Wars agent evaluation.

This module provides the EssenceWarsBenchmark class for standardized
evaluation of agents against multiple baselines.

Example:
    from essence_wars.benchmark import EssenceWarsBenchmark, NeuralAgent

    # Create benchmark
    benchmark = EssenceWarsBenchmark(games_per_opponent=100)

    # Load your trained agent
    agent = NeuralAgent.from_checkpoint("models/my_model.pt")

    # Run standard evaluation
    results = benchmark.evaluate(agent)
    print(results.summary())

    # Save results
    results.save("benchmark_results.json")
"""

from __future__ import annotations

import time
from typing import TYPE_CHECKING

import numpy as np

if TYPE_CHECKING:
    from essence_wars._core import PyGame

from .agents import BenchmarkAgent
from .elo import EloTracker
from .metrics import (
    BenchmarkResults,
    DeckResult,
    GeneralizationResults,
    MatchupResult,
    TransferResults,
)


class EssenceWarsBenchmark:
    """Standardized benchmark for Essence Wars agents.

    Evaluates agents against standard baselines (Random, Greedy, MCTS)
    and computes Elo ratings for fair comparison.

    Features:
    - Standard evaluation suite with 4 baselines
    - Elo rating computation
    - Per-deck performance breakdown
    - Transfer learning evaluation
    - Cross-faction generalization testing
    - JSON export for reproducibility

    Example:
        benchmark = EssenceWarsBenchmark()

        # Quick evaluation (fewer games)
        results = benchmark.evaluate(agent, games_per_opponent=50)

        # Full evaluation
        results = benchmark.evaluate(agent, games_per_opponent=200)

        # Get Elo rating
        print(f"Elo: {results.elo_rating}")
    """

    # Standard baselines for evaluation
    BASELINES = ["random", "greedy", "mcts50", "mcts100"]

    # Available decks (loaded dynamically)
    DECKS: list[str] = []

    def __init__(
        self,
        games_per_opponent: int = 100,
        game_mode: str = "attrition",
        verbose: bool = True,
    ):
        """Initialize benchmark.

        Args:
            games_per_opponent: Games to play against each baseline
            game_mode: "attrition" or "essence_duel"
            verbose: Print progress during evaluation
        """
        self.games_per_opponent = games_per_opponent
        self.game_mode = game_mode
        self.verbose = verbose

        # Initialize Elo tracker with baseline ratings
        self.elo = EloTracker(k_factor=32, initial_rating=1500)
        self._init_baseline_ratings()

        # Load available decks
        self._load_decks()

    def _init_baseline_ratings(self) -> None:
        """Set initial Elo ratings for baselines based on empirical data."""
        # These are approximate ratings based on typical matchups
        self.elo.ratings["random"] = self.elo.get_agent_stats("random")
        self.elo.ratings["random"].rating = 1000

        self.elo.ratings["greedy"] = self.elo.get_agent_stats("greedy")
        self.elo.ratings["greedy"].rating = 1300

        self.elo.ratings["mcts50"] = self.elo.get_agent_stats("mcts50")
        self.elo.ratings["mcts50"].rating = 1450

        self.elo.ratings["mcts100"] = self.elo.get_agent_stats("mcts100")
        self.elo.ratings["mcts100"].rating = 1500

    def _load_decks(self) -> None:
        """Load available deck IDs from the game."""
        try:
            from essence_wars._core import PyGame
            self.DECKS = PyGame.list_decks()
        except (ImportError, AttributeError):
            # Fallback to known decks
            self.DECKS = [
                "argentum_control", "argentum_midrange",
                "symbiote_aggro", "symbiote_tempo",
                "obsidion_burst", "obsidion_control",
            ]

    def _log(self, msg: str) -> None:
        """Print message if verbose mode."""
        if self.verbose:
            print(msg)

    def evaluate(
        self,
        agent: BenchmarkAgent,
        games_per_opponent: int | None = None,
        baselines: list[str] | None = None,
    ) -> BenchmarkResults:
        """Run standard evaluation suite.

        Evaluates the agent against Random, Greedy, MCTS-50, and MCTS-100
        baselines, computing win rates and Elo rating.

        Args:
            agent: Agent to evaluate (must implement BenchmarkAgent protocol)
            games_per_opponent: Override default games per opponent
            baselines: Override default baselines (e.g., ["random", "greedy"])

        Returns:
            BenchmarkResults with all metrics
        """

        games_per = games_per_opponent or self.games_per_opponent
        baselines = baselines or self.BASELINES

        self._log(f"\n=== Evaluating: {agent.name} ===")
        self._log(f"Games per opponent: {games_per}")
        self._log(f"Baselines: {baselines}")
        self._log("")

        results = BenchmarkResults(
            agent_name=agent.name,
            game_mode=self.game_mode,
            games_per_opponent=games_per,
        )

        total_games = 0
        total_wins = 0
        total_game_lengths = []
        total_decision_times = []

        # Evaluate against each baseline
        for baseline in baselines:
            self._log(f"vs {baseline}...")

            matchup = self._evaluate_matchup(
                agent, baseline, games_per
            )
            results.matchup_results.append(matchup)

            # Update aggregates
            total_games += matchup.games_played
            total_wins += matchup.wins
            total_game_lengths.append(matchup.avg_game_length)
            total_decision_times.append(matchup.avg_decision_time_ms)

            # Update Elo
            for _ in range(matchup.wins):
                self.elo.update(agent.name, baseline, winner=0)
            for _ in range(matchup.losses):
                self.elo.update(agent.name, baseline, winner=1)
            for _ in range(matchup.draws):
                self.elo.update(agent.name, baseline, winner=-1)

            # Store win rate
            if baseline == "random":
                results.win_rate_vs_random = matchup.win_rate
            elif baseline == "greedy":
                results.win_rate_vs_greedy = matchup.win_rate
            elif baseline == "mcts50":
                results.win_rate_vs_mcts50 = matchup.win_rate
            elif baseline == "mcts100":
                results.win_rate_vs_mcts100 = matchup.win_rate

            self._log(f"  Win rate: {matchup.win_rate:.1%}")

        # Compute aggregates
        results.total_games = total_games
        results.total_wins = total_wins
        results.avg_game_length = float(np.mean(total_game_lengths)) if total_game_lengths else 0.0
        results.avg_decision_time_ms = float(np.mean(total_decision_times)) if total_decision_times else 0.0
        results.elo_rating = self.elo.get_rating(agent.name)

        self._log(f"\nFinal Elo: {results.elo_rating:.0f}")

        return results

    def _evaluate_matchup(
        self,
        agent: BenchmarkAgent,
        opponent: str,
        num_games: int,
    ) -> MatchupResult:
        """Evaluate agent against a single opponent type."""
        from essence_wars._core import PyGame

        wins = 0
        losses = 0
        draws = 0
        game_lengths = []
        decision_times = []

        for game_idx in range(num_games):
            # Select random decks for variety
            deck1 = np.random.choice(self.DECKS) if self.DECKS else "argentum_control"
            deck2 = np.random.choice(self.DECKS) if self.DECKS else "symbiote_aggro"

            # Create game
            game = PyGame(
                deck1=deck1,
                deck2=deck2,
                game_mode=self.game_mode,
            )
            game.reset(seed=game_idx + 1000)

            # Play game
            agent.reset()
            turns = 0
            agent_times = []

            while not game.is_done():
                current_player = game.current_player()
                obs = np.array(game.observe(), dtype=np.float32)
                mask = np.array(game.action_mask(), dtype=np.float32)

                if current_player == 0:
                    # Agent's turn
                    start = time.perf_counter()
                    action = agent.select_action(obs, mask)
                    elapsed = (time.perf_counter() - start) * 1000
                    agent_times.append(elapsed)
                else:
                    # Opponent's turn
                    action = self._get_opponent_action(game, opponent, obs, mask)

                game.step(action)
                turns += 1

                # Safety limit
                if turns > 500:
                    break

            # Determine winner from reward
            # get_reward returns +1 for win, -1 for loss, 0 for draw
            reward = game.get_reward(0)  # Player 0 (our agent)
            if reward > 0:
                wins += 1
            elif reward < 0:
                losses += 1
            else:
                draws += 1

            game_lengths.append(turns)
            if agent_times:
                decision_times.extend(agent_times)

        return MatchupResult(
            opponent=opponent,
            games_played=num_games,
            wins=wins,
            losses=losses,
            draws=draws,
            avg_game_length=float(np.mean(game_lengths)) if game_lengths else 0.0,
            avg_decision_time_ms=float(np.mean(decision_times)) if decision_times else 0.0,
        )

    def _get_opponent_action(
        self,
        game: PyGame,
        opponent: str,
        obs: np.ndarray,
        mask: np.ndarray,
    ) -> int:
        """Get action from opponent bot."""
        if opponent == "random":
            valid = np.where(mask > 0.5)[0]
            return int(np.random.choice(valid)) if len(valid) > 0 else 255

        elif opponent == "greedy":
            # Use Rust greedy bot
            action: int = game.greedy_action()
            return action

        elif opponent.startswith("mcts"):
            # Parse simulation count
            sims = int(opponent.replace("mcts", ""))
            action = game.mcts_action(sims)
            return int(action)

        else:
            raise ValueError(f"Unknown opponent: {opponent}")

    def evaluate_per_deck(
        self,
        agent: BenchmarkAgent,
        opponent: str = "greedy",
        games_per_deck: int = 50,
    ) -> list[DeckResult]:
        """Evaluate agent performance for each deck.

        Plays games with the agent using each available deck
        to identify deck-specific strengths and weaknesses.

        Args:
            agent: Agent to evaluate
            opponent: Opponent type for evaluation
            games_per_deck: Games to play with each deck

        Returns:
            List of DeckResult for each deck
        """
        from essence_wars._core import PyGame

        self._log(f"\n=== Per-Deck Evaluation: {agent.name} ===")
        self._log(f"Opponent: {opponent}")
        self._log(f"Games per deck: {games_per_deck}")
        self._log("")

        results = []

        for deck in self.DECKS:
            self._log(f"  {deck}...")

            wins = 0
            games = 0

            for game_idx in range(games_per_deck):
                # Agent always uses the target deck
                opponent_deck = np.random.choice(
                    [d for d in self.DECKS if d != deck]
                ) if len(self.DECKS) > 1 else deck

                game = PyGame(
                    deck1=deck,
                    deck2=opponent_deck,
                    game_mode=self.game_mode,
                )
                game.reset(seed=game_idx + 2000)

                agent.reset()
                turns = 0

                while not game.is_done() and turns < 500:
                    current_player = game.current_player()
                    obs = np.array(game.observe(), dtype=np.float32)
                    mask = np.array(game.action_mask(), dtype=np.float32)

                    if current_player == 0:
                        action = agent.select_action(obs, mask)
                    else:
                        action = self._get_opponent_action(game, opponent, obs, mask)

                    game.step(action)
                    turns += 1

                reward = game.get_reward(0)
                if reward > 0:
                    wins += 1
                games += 1

            win_rate = wins / games if games > 0 else 0
            results.append(DeckResult(
                deck_id=deck,
                games_played=games,
                wins=wins,
                win_rate=win_rate,
            ))
            self._log(f"    {win_rate:.1%}")

        return results

    def evaluate_transfer(
        self,
        agent: BenchmarkAgent,
        train_mode: str,
        test_mode: str,
        opponent: str = "greedy",
        num_games: int = 100,
    ) -> TransferResults:
        """Evaluate transfer learning between game modes.

        Tests how well an agent trained on one game mode performs
        on a different game mode.

        Args:
            agent: Agent to evaluate
            train_mode: Mode the agent was trained on
            test_mode: Mode to test on
            opponent: Opponent type
            num_games: Games to play

        Returns:
            TransferResults with win rate on test mode
        """
        from essence_wars._core import PyGame

        self._log(f"\n=== Transfer Evaluation: {train_mode} -> {test_mode} ===")

        wins = 0

        for game_idx in range(num_games):
            deck1 = np.random.choice(self.DECKS) if self.DECKS else "argentum_control"
            deck2 = np.random.choice(self.DECKS) if self.DECKS else "symbiote_aggro"

            game = PyGame(
                deck1=deck1,
                deck2=deck2,
                game_mode=test_mode,  # Use test mode
            )
            game.reset(seed=game_idx + 3000)

            agent.reset()
            turns = 0

            while not game.is_done() and turns < 500:
                current_player = game.current_player()
                obs = np.array(game.observe(), dtype=np.float32)
                mask = np.array(game.action_mask(), dtype=np.float32)

                if current_player == 0:
                    action = agent.select_action(obs, mask)
                else:
                    action = self._get_opponent_action(game, opponent, obs, mask)

                game.step(action)
                turns += 1

            if game.get_reward(0) > 0:
                wins += 1

        win_rate = wins / num_games
        self._log(f"Win rate: {win_rate:.1%}")

        return TransferResults(
            agent_name=agent.name,
            train_mode=train_mode,
            test_mode=test_mode,
            win_rate=win_rate,
            games_played=num_games,
        )

    def evaluate_generalization(
        self,
        agent: BenchmarkAgent,
        train_factions: list[str],
        test_factions: list[str],
        opponent: str = "greedy",
        games_per_faction: int = 50,
    ) -> GeneralizationResults:
        """Evaluate cross-faction generalization.

        Tests how well an agent trained on some factions performs
        when playing factions it hasn't seen.

        Args:
            agent: Agent to evaluate
            train_factions: Factions used during training
            test_factions: Unseen factions to test on
            opponent: Opponent type
            games_per_faction: Games per faction

        Returns:
            GeneralizationResults with train/test win rates
        """
        from essence_wars._core import PyGame

        self._log(f"\n=== Generalization: {train_factions} -> {test_factions} ===")

        def get_faction_decks(factions: list[str]) -> list[str]:
            """Get decks belonging to specified factions."""
            faction_decks = []
            for deck in self.DECKS:
                for faction in factions:
                    if faction.lower() in deck.lower():
                        faction_decks.append(deck)
                        break
            return faction_decks

        train_decks = get_faction_decks(train_factions)
        test_decks = get_faction_decks(test_factions)

        def evaluate_on_decks(decks: list[str]) -> float:
            if not decks:
                return 0.0

            wins = 0
            games = 0

            for deck in decks:
                for game_idx in range(games_per_faction):
                    opponent_deck = np.random.choice(self.DECKS)

                    game = PyGame(
                        deck1=deck,
                        deck2=opponent_deck,
                        game_mode=self.game_mode,
                    )
                    game.reset(seed=game_idx + 4000 + games)

                    agent.reset()
                    turns = 0

                    while not game.is_done() and turns < 500:
                        current_player = game.current_player()
                        obs = np.array(game.observe(), dtype=np.float32)
                        mask = np.array(game.action_mask(), dtype=np.float32)

                        if current_player == 0:
                            action = agent.select_action(obs, mask)
                        else:
                            action = self._get_opponent_action(game, opponent, obs, mask)

                        game.step(action)
                        turns += 1

                    if game.get_reward(0) > 0:
                        wins += 1
                    games += 1

            return wins / games if games > 0 else 0.0

        win_rate_train = evaluate_on_decks(train_decks)
        win_rate_test = evaluate_on_decks(test_decks)

        self._log(f"Train factions ({train_factions}): {win_rate_train:.1%}")
        self._log(f"Test factions ({test_factions}): {win_rate_test:.1%}")

        total_games = len(train_decks + test_decks) * games_per_faction

        return GeneralizationResults(
            agent_name=agent.name,
            train_factions=train_factions,
            test_factions=test_factions,
            win_rate_train=win_rate_train,
            win_rate_test=win_rate_test,
            games_played=total_games,
        )

    def quick_evaluate(
        self,
        agent: BenchmarkAgent,
        games: int = 20,
    ) -> dict[str, float]:
        """Quick evaluation with fewer games.

        Useful for fast iteration during training.

        Args:
            agent: Agent to evaluate
            games: Total games to play (split across baselines)

        Returns:
            Dictionary with win rates
        """
        results = self.evaluate(
            agent,
            games_per_opponent=games // 2,
            baselines=["random", "greedy"],
        )
        return {
            "vs_random": results.win_rate_vs_random,
            "vs_greedy": results.win_rate_vs_greedy,
            "elo": results.elo_rating,
        }
