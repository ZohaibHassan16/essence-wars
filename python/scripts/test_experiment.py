
import os
import sys

# Add project root to path so we can import cardgame
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '../../')))

from cardgame.infra import Experiment


def main():
    print("Initializing test experiment...")

    config = {
        "learning_rate": 0.001,
        "batch_size": 64,
        "model": "transformer_v1"
    }

    exp = Experiment("test_algo", config, tag="sanity_check")

    print(f"Experiment created at: {exp.get_dir()}")

    # Simulate some logging
    exp.log("Starting training loop...")
    metrics = []
    for i in range(5):
        metrics.append({
            "step": i,
            "loss": 1.0 - (i * 0.1),
            "accuracy": 0.5 + (i * 0.1)
        })
        exp.log(f"Step {i}: loss={metrics[-1]['loss']}")

    exp.save_metrics(metrics)
    print("Metrics saved.")

    print("Test complete.")

if __name__ == "__main__":
    main()
