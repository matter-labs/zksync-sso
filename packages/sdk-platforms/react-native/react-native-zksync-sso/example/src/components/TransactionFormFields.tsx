import React from 'react';
import {
    View,
    Text,
    TextInput,
    StyleSheet,
    TouchableOpacity
} from 'react-native';
import AddressView from './AddressView';

interface TransactionFormFieldsProps {
    fromAddress: string;
    toAddress: string;
    setToAddress: (value: string) => void;
    amount: string;
    setAmount: (value: string) => void;
}

const TransactionFormFields: React.FC<TransactionFormFieldsProps> = ({
    fromAddress,
    toAddress,
    setToAddress,
    amount,
    setAmount
}) => {
    return (
        <View style={styles.container}>
            <View style={styles.section}>
                <Text style={styles.sectionTitle}>From</Text>
                <View style={styles.field}>
                    <AddressView address={fromAddress} />
                </View>
            </View>

            <View style={styles.section}>
                <Text style={styles.sectionTitle}>To</Text>
                <View style={styles.inputContainer}>
                    <TextInput
                        style={styles.monospaceInput}
                        placeholder="Recipient Address"
                        value={toAddress}
                        onChangeText={setToAddress}
                        autoCapitalize="none"
                        autoCorrect={false}
                    />
                    {toAddress.length > 0 && (
                        <TouchableOpacity
                            onPress={() => setToAddress('')}
                            style={styles.clearButton}
                        >
                            <Text style={styles.clearButtonText}>✕</Text>
                        </TouchableOpacity>
                    )}
                </View>
            </View>

            <View style={styles.section}>
                <Text style={styles.sectionTitle}>Amount</Text>
                <View style={styles.inputContainer}>
                    <TextInput
                        style={styles.input}
                        placeholder="Amount in ETH"
                        value={amount}
                        onChangeText={setAmount}
                        keyboardType="decimal-pad"
                    />
                    {amount.length > 0 && (
                        <TouchableOpacity
                            onPress={() => setAmount('')}
                            style={styles.clearButton}
                        >
                            <Text style={styles.clearButtonText}>✕</Text>
                        </TouchableOpacity>
                    )}
                </View>
            </View>
        </View>
    );
};

const styles = StyleSheet.create({
    container: {
        flex: 1,
    },
    section: {
        marginBottom: 20,
    },
    sectionTitle: {
        fontSize: 17,
        fontWeight: '600',
        marginBottom: 8,
        color: '#8E8E93',
    },
    field: {
        marginBottom: 8,
    },
    inputContainer: {
        flexDirection: 'row',
        alignItems: 'center',
        borderRadius: 8,
        backgroundColor: '#F2F2F7',
        paddingHorizontal: 12,
    },
    input: {
        flex: 1,
        height: 50,
        fontSize: 16,
    },
    monospaceInput: {
        flex: 1,
        height: 50,
        fontSize: 16,
        fontFamily: 'monospace',
    },
    clearButton: {
        padding: 10,
        borderRadius: 15,
    },
    clearButtonText: {
        color: '#888',
        fontSize: 16,
        fontWeight: 'bold',
    }
});

export default TransactionFormFields; 