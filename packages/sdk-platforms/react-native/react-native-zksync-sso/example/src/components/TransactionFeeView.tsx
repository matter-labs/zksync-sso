import React from 'react';
import { View, Text, StyleSheet, ActivityIndicator } from 'react-native';

interface TransactionFeeViewProps {
    fee: string;
    isPreparing: boolean;
}

const TransactionFeeView: React.FC<TransactionFeeViewProps> = ({
    fee,
    isPreparing
}) => {
    return (
        <View style={styles.container}>
            <Text style={styles.title}>Network Fee</Text>
            <View style={styles.feeContainer}>
                {isPreparing ? (
                    <ActivityIndicator size="small" color="#888" />
                ) : (
                    <Text style={styles.feeText}>{fee}</Text>
                )}
                <Text style={styles.explainerText}>
                    This is the fee paid to miners to process your transaction
                </Text>
            </View>
        </View>
    );
};

const styles = StyleSheet.create({
    container: {
        marginBottom: 20,
    },
    title: {
        fontSize: 17,
        fontWeight: '600',
        marginBottom: 8,
        color: '#8E8E93',
    },
    feeContainer: {
        padding: 16,
        backgroundColor: 'rgba(150, 150, 150, 0.1)',
        borderRadius: 8,
    },
    feeText: {
        fontSize: 16,
        fontWeight: 'bold',
        marginBottom: 8,
    },
    explainerText: {
        fontSize: 14,
        color: '#8E8E93',
    }
});

export default TransactionFeeView; 