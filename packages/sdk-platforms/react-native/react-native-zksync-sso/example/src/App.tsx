import { View, StyleSheet } from 'react-native';
import ExampleView from './components/ExampleView';

export default function App() {
  return (
    <View style={styles.container}>
      <ExampleView relyingPartyIdentifier="soo-sdk-example-pages.pages.dev" />
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    width: '100%',
  },
});
