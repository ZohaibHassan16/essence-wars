import os
import json
import yaml
import time
import logging
from pathlib import Path
from datetime import datetime
from typing import Any, Dict, Optional, Union

class Experiment:
    """
    Manages experiment directory creation, configuration saving, and logging.
    
    Usage:
        exp = Experiment("ppo", {"lr": 3e-4, "steps": 1000}, tag="baseline")
        exp.log_metric("loss", 0.5, step=1)
        path = exp.get_path("checkpoints/model.pt")
    """
    
    def __init__(self, 
                 algo_name: str, 
                 config: Dict[str, Any], 
                 tag: str = "default", 
                 root_dir: str = "experiments"):
        
        self.timestamp = datetime.now().strftime("%Y-%m-%d_%H-%M-%S")
        self.name = f"{self.timestamp}_{tag}"
        self.algo_name = algo_name
        self.root_dir = Path(root_dir)
        
        # Create directory structure
        self.exp_dir = self.root_dir / algo_name / self.name
        self.exp_dir.mkdir(parents=True, exist_ok=True)
        
        # Subdirectories
        (self.exp_dir / "checkpoints").mkdir(exist_ok=True)
        (self.exp_dir / "plots").mkdir(exist_ok=True)
        (self.exp_dir / "logs").mkdir(exist_ok=True)
        
        # Setup logging
        self._setup_logging()
        
        # Save config
        self.save_config(config)
        
        self.logger.info(f"Experiment initialized: {self.exp_dir}")
        
    def _setup_logging(self):
        self.logger = logging.getLogger(self.name)
        self.logger.setLevel(logging.INFO)
        
        # File handler
        fh = logging.FileHandler(self.exp_dir / "logs" / "run.log")
        fh.setFormatter(logging.Formatter('%(asctime)s - %(levelname)s - %(message)s'))
        self.logger.addHandler(fh)
        
        # Console handler
        ch = logging.StreamHandler()
        ch.setFormatter(logging.Formatter('%(message)s'))
        self.logger.addHandler(ch)

    def save_config(self, config: Dict[str, Any]):
        """Save configuration to yaml."""
        with open(self.exp_dir / "config.yaml", 'w') as f:
            yaml.dump(config, f, default_flow_style=False)

    def get_dir(self) -> Path:
        """Get experiment root directory."""
        return self.exp_dir

    def log(self, msg: str):
        self.logger.info(msg)
        
    def save_metrics(self, metrics: list[dict], filename: str = "metrics.csv"):
        """Save a list of metric dicts to CSV."""
        if not metrics:
            return
            
        import csv
        path = self.exp_dir / filename
        
        # Determine write mode (overwrite or append)
        mode = 'w'
        
        with open(path, mode, newline='') as f:
            writer = csv.DictWriter(f, fieldnames=metrics[0].keys())
            writer.writeheader()
            writer.writerows(metrics)
            
    def get_checkpoint_path(self, name: str) -> Path:
        return self.exp_dir / "checkpoints" / name
