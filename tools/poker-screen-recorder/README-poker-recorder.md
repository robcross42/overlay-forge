# Poker Screen Recorder

Passive full-screen screenshot recorder for learning and post-session review.

## Run

```powershell
python -m pip install pillow
python poker_screen_recorder.py
```

Use **Capture now** for a manual PNG capture. Enable the interval checkbox for periodic full-screen PNG captures. Configure the output folder and interval in `recorder_config.json`; captures default to the ignored `captures/` folder beside the script.

The recorder intentionally does not read card values, analyze hands, inspect a poker client, control another application, or provide real-time recommendations.
