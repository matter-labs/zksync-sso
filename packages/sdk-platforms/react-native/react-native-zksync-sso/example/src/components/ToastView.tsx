import React, { useEffect } from 'react';
import { View, Text, StyleSheet, Animated } from 'react-native';

interface ToastViewProps {
    message: string;
    icon?: string;
    iconColor?: string;
}

const ToastView: React.FC<ToastViewProps> = ({
    message,
    icon = '✓',
    iconColor = '#4CD964'
}) => {
    const opacity = React.useRef(new Animated.Value(0)).current;
    const scale = React.useRef(new Animated.Value(0.8)).current;

    useEffect(() => {
        Animated.parallel([
            Animated.timing(opacity, {
                toValue: 1,
                duration: 300,
                useNativeDriver: true,
            }),
            Animated.timing(scale, {
                toValue: 1,
                duration: 300,
                useNativeDriver: true,
            }),
        ]).start();
        
        const timer = setTimeout(() => {
            Animated.parallel([
                Animated.timing(opacity, {
                    toValue: 0,
                    duration: 300,
                    useNativeDriver: true,
                }),
                Animated.timing(scale, {
                    toValue: 0.8,
                    duration: 300,
                    useNativeDriver: true,
                }),
            ]).start();
        }, 2000);

        return () => clearTimeout(timer);
    }, []);

    return (
        <Animated.View
            style={[
                styles.container,
                {
                    opacity,
                    transform: [{ scale }],
                },
            ]}
        >
            <View style={styles.content}>
                <Text style={[styles.icon, { color: iconColor }]}>{icon}</Text>
                <Text style={styles.message}>{message}</Text>
            </View>
        </Animated.View>
    );
};

const styles = StyleSheet.create({
    container: {
        position: 'absolute',
        top: 0,
        left: 0,
        right: 0,
        bottom: 0,
        justifyContent: 'center',
        alignItems: 'center',
        pointerEvents: 'none',
    },
    content: {
        alignItems: 'center',
        backgroundColor: 'rgba(240, 240, 240, 0.85)',
        paddingVertical: 32,
        paddingHorizontal: 32,
        borderRadius: 16,
        maxWidth: 280,
        shadowColor: '#000',
        shadowOffset: { width: 0, height: 2 },
        shadowOpacity: 0.1,
        shadowRadius: 4,
        elevation: 4,
    },
    icon: {
        fontSize: 48,
        marginBottom: 12,
    },
    message: {
        fontSize: 17,
        fontWeight: '600',
        textAlign: 'center',
    },
});

export default ToastView; 