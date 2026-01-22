#!/usr/bin/env python3
"""
Extract Canny edges from image for ControlNet guidance.
AI agent friendly CLI tool.
"""
import sys
import argparse
from pathlib import Path

def extract_edges_opencv(input_path, output_path, low_threshold=100, high_threshold=200):
    """Extract edges using OpenCV (preferred method)."""
    try:
        import cv2
        import numpy as np
    except ImportError:
        print("ERROR: OpenCV not installed. Run: pip install opencv-python", file=sys.stderr)
        return 1
    
    img = cv2.imread(str(input_path), cv2.IMREAD_GRAYSCALE)
    if img is None:
        print(f"ERROR: Could not read {input_path}", file=sys.stderr)
        return 1
    
    edges = cv2.Canny(img, low_threshold, high_threshold)
    
    # Convert grayscale edges to RGB (ControlNet needs 3 channels)
    edges_rgb = cv2.cvtColor(edges, cv2.COLOR_GRAY2RGB)
    
    success = cv2.imwrite(str(output_path), edges_rgb)
    
    if success:
        print(f"SUCCESS: Edges saved to {output_path}")
        return 0
    else:
        print(f"ERROR: Failed to write {output_path}", file=sys.stderr)
        return 1

def extract_edges_imagemagick(input_path, output_path):
    """Extract edges using ImageMagick (fallback)."""
    import subprocess
    from pathlib import Path
    
    # Create temp grayscale edge file
    temp_edges = Path(output_path).parent / f"temp_edges_{Path(output_path).stem}.png"
    
    # Step 1: Extract edges (creates grayscale)
    cmd1 = ["convert", str(input_path), "-canny", "0x1+10%+30%", str(temp_edges)]
    
    # Step 2: Convert grayscale to RGB (ControlNet needs 3 channels)
    # Use -separate -combine to duplicate grayscale channel to R, G, B
    cmd2 = ["convert", str(temp_edges), "-separate", "-combine", f"PNG24:{output_path}"]
    
    try:
        result1 = subprocess.run(cmd1, capture_output=True, text=True)
        if result1.returncode != 0:
            print(f"ERROR: ImageMagick edge extraction failed: {result1.stderr}", file=sys.stderr)
            return 1
        
        result2 = subprocess.run(cmd2, capture_output=True, text=True)
        if result2.returncode != 0:
            print(f"ERROR: ImageMagick RGB conversion failed: {result2.stderr}", file=sys.stderr)
            return 1
        
        # Clean up temp file
        if temp_edges.exists():
            temp_edges.unlink()
        
        print(f"SUCCESS: RGB edges saved to {output_path}")
        return 0
    except FileNotFoundError:
        print("ERROR: ImageMagick not installed. Run: sudo apt-get install imagemagick", file=sys.stderr)
        return 1

def main():
    parser = argparse.ArgumentParser(
        description="Extract Canny edges for FLUX ControlNet guidance",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Basic usage (OpenCV)
  %(prog)s reference.png edges.png
  
  # Adjust sensitivity (OpenCV only)
  %(prog)s reference.png edges.png --low 50 --high 150
  
  # Use ImageMagick instead
  %(prog)s reference.png edges.png --method imagemagick
        """
    )
    
    parser.add_argument("input", type=Path, help="Input image path")
    parser.add_argument("output", type=Path, help="Output edge map path")
    parser.add_argument("--low", type=int, default=100, 
                       help="Low threshold for Canny (default: 100, OpenCV only)")
    parser.add_argument("--high", type=int, default=200, 
                       help="High threshold for Canny (default: 200, OpenCV only)")
    parser.add_argument("--method", choices=["opencv", "imagemagick"], default="opencv",
                       help="Edge detection method (default: opencv)")
    
    args = parser.parse_args()
    
    # Validate input file exists
    if not args.input.exists():
        print(f"ERROR: Input file not found: {args.input}", file=sys.stderr)
        return 1
    
    # Create output directory if needed
    args.output.parent.mkdir(parents=True, exist_ok=True)
    
    # Extract edges using chosen method
    if args.method == "opencv":
        return extract_edges_opencv(args.input, args.output, args.low, args.high)
    else:
        return extract_edges_imagemagick(args.input, args.output)

if __name__ == "__main__":
    sys.exit(main())
