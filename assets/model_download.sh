#!/bin/sh

echo "Please note: These models are licensed under CC BY-SA 4.0 by the ocrs-models project."

DETECTION_MODEL="https://ocrs-models.s3-accelerate.amazonaws.com/text-detection.rten"
RECOGNITION_MODEL="https://ocrs-models.s3-accelerate.amazonaws.com/text-recognition.rten"

curl "$DETECTION_MODEL" -o text-detection.rten
curl "$RECOGNITION_MODEL" -o text-recognition.rten
