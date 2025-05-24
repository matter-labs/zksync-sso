import React from 'react';
import { View, StyleSheet } from 'react-native';
import ExampleView from './components/ExampleView';
import { initializePlatformLogger } from '../../src/passkey/utils';
import { RpIdHelper } from './components/types';

export default function App() {
  // Initialize platform-specific logging before any SDK usage
  initializePlatformLogger("io.jackpooley.MLSSOExampleRN");

  const rpId = RpIdHelper.createForCurrentPlatform(
    "soo-sdk-example-pages.pages.dev", // RP ID (same for both platforms)
    "android:apk-key-hash:-sYXRdwJA3hvue3mKpYrOZ9zSPC7b4mbgzJmdZEDO5w" // Android origin
  );

  return (
    <View style={styles.container}>
      <ExampleView rpId={rpId} />
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    width: '100%',
  },
});
