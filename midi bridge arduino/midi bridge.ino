const int buttonPin = 2;       // Digital pin for button
const int pedalPin = A0;       // Analog pin for exp. pedal

byte lastValue = 255;          // Last MIDI value

void setup() {
  Serial.begin(115200);
  pinMode(buttonPin, INPUT_PULLUP);
  delay(1000);

  // Device ID for the program to find
  Serial.write(0xFA);
  Serial.write(0xCE);
}

// A simple debounce
bool isButtonPressed() {
  static bool lastState = HIGH;
  bool reading = digitalRead(buttonPin);

  if (reading != lastState) {
    delay(5);
    reading = digitalRead(buttonPin);
  }

  lastState = reading;
  return reading == LOW;
}

void loop() {
  
  bool buttonPressed = isButtonPressed();

  // Read analog value
  int rawAnalog = analogRead(pedalPin);

  
  if (rawAnalog < 5 || rawAnalog > 1018) {
    delay(50);
    return;
  }

  // convert into 0-127 range
  byte pedalValue = map(rawAnalog, 0, 1023, 0, 127);

  
  byte midiValue = buttonPressed ? 127 : pedalValue;

  // Send only if there is a change
  if (midiValue != lastValue) {
    Serial.write(0xB0);  // Control Change message (channel 1)
    Serial.write(99);    // CC99
    Serial.write(midiValue);
    lastValue = midiValue;
  }

  delay(20); // To not send too much data
}
