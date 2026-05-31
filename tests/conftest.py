import sys
import os

# Add output directory to path for TestModule import
_output_path = os.path.join(os.path.dirname(__file__), "output")
if _output_path not in sys.path:
    sys.path.insert(0, _output_path)
