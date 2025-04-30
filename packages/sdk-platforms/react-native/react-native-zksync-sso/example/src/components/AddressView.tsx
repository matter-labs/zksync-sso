import React, { useState, useEffect } from 'react';
import {
    View,
    Text,
    StyleSheet,
    TouchableOpacity,
    Animated,
    Clipboard
} from 'react-native';

interface AddressViewProps {
    address: string;
}

const AddressView: React.FC<AddressViewProps> = ({ address }) => {
    const [showingCopiedFeedback, setShowingCopiedFeedback] = useState(false);
    const opacityAnim = useState(new Animated.Value(0))[0];
    const translateYAnim = useState(new Animated.Value(-20))[0];

    useEffect(() => {
        if (showingCopiedFeedback) {
            Animated.parallel([
                Animated.timing(opacityAnim, {
                    toValue: 1,
                    duration: 150,
                    useNativeDriver: true,
                }),
                Animated.timing(translateYAnim, {
                    toValue: 0,
                    duration: 150,
                    useNativeDriver: true,
                }),
            ]).start();

            const timer = setTimeout(() => {
                hideFeedback();
            }, 2000);

            return () => clearTimeout(timer);
        } else {
            Animated.parallel([
                Animated.timing(opacityAnim, {
                    toValue: 0,
                    duration: 150,
                    useNativeDriver: true,
                }),
                Animated.timing(translateYAnim, {
                    toValue: -20,
                    duration: 150,
                    useNativeDriver: true,
                }),
            ]).start();
        }
    }, [showingCopiedFeedback]);

    const handleCopy = () => {
        Clipboard.setString(address);
        console.log('Copied to clipboard:', address);
        setShowingCopiedFeedback(true);
    };

    const hideFeedback = () => {
        setShowingCopiedFeedback(false);
    };

    return (
        <View style={styles.container}>
            <TouchableOpacity
                style={styles.addressContainer}
                onPress={handleCopy}
                activeOpacity={0.7}
            >
                <Text style={styles.addressText} numberOfLines={1} ellipsizeMode="middle">
                    {address}
                </Text>
            </TouchableOpacity>

            <Animated.View
                style={[
                    styles.feedbackContainer,
                    {
                        opacity: opacityAnim,
                        transform: [{ translateY: translateYAnim }],
                    },
                ]}
                pointerEvents="none"
            >
                <Text style={styles.feedbackText}>Copied!</Text>
            </Animated.View>
        </View>
    );
};

const styles = StyleSheet.create({
    container: {
        position: 'relative',
    },
    addressContainer: {
        padding: 16,
        backgroundColor: 'rgba(150, 150, 150, 0.1)',
        borderRadius: 12,
    },
    addressText: {
        fontSize: 16,
        color: '#000000',
    },
    feedbackContainer: {
        position: 'absolute',
        top: -30,
        alignSelf: 'center',
        backgroundColor: 'rgba(240, 240, 240, 0.9)',
        borderRadius: 4,
        paddingVertical: 4,
        paddingHorizontal: 8,
    },
    feedbackText: {
        fontSize: 12,
        color: '#666666',
    },
});

export default AddressView; 