package org.mensetsu.app.core.theme

import androidx.compose.material3.Typography
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.sp

val PrimaryIndigo = Color(0xFF4F46E5)
val PrimaryLightIndigo = Color(0xFFEEF2FF)
val PrimaryDarkIndigo = Color(0xFF3730A3)

val Slate900 = Color(0xFF0F172A)
val Slate700 = Color(0xFF334155)
val Slate600 = Color(0xFF475569)
val Slate500 = Color(0xFF64748B)
val Slate400 = Color(0xFF94A3B8)
val Slate200 = Color(0xFFE2E8F0)
val Slate100 = Color(0xFFF1F5F9)

val EmeraldGreen = Color(0xFF10B981)
val RoseRed = Color(0xFFF43F5E)

val LightColorScheme = lightColorScheme(
    primary = PrimaryIndigo,
    onPrimary = Color.White,
    primaryContainer = PrimaryLightIndigo,
    onPrimaryContainer = PrimaryDarkIndigo,
    secondary = Slate700,
    onSecondary = Color.White,
    background = Color(0xFFF8FAFC),
    onBackground = Slate900,
    surface = Color.White,
    onSurface = Slate900,
    surfaceVariant = Slate100,
    onSurfaceVariant = Slate700,
    outline = Slate200
)

val DarkColorScheme = darkColorScheme(
    primary = Color(0xFF818CF8),
    onPrimary = Color.Black,
    primaryContainer = Color(0xFF312E81),
    onPrimaryContainer = Color(0xFFEEF2FF),
    secondary = Slate400,
    onSecondary = Color.Black,
    background = Color(0xFF0B0F19),
    onBackground = Color(0xFFF1F5F9),
    surface = Color(0xFF111827),
    onSurface = Color(0xFFF1F5F9),
    surfaceVariant = Color(0xFF1F2937),
    onSurfaceVariant = Color(0xFF9CA3AF),
    outline = Color(0xFF374151)
)

val AppTypography = Typography(
    headlineLarge = TextStyle(
        fontFamily = FontFamily.Default,
        fontWeight = FontWeight.Bold,
        fontSize = 24.sp,
        lineHeight = 32.sp
    ),
    headlineMedium = TextStyle(
        fontFamily = FontFamily.Default,
        fontWeight = FontWeight.SemiBold,
        fontSize = 20.sp,
        lineHeight = 28.sp
    ),
    titleLarge = TextStyle(
        fontFamily = FontFamily.Default,
        fontWeight = FontWeight.SemiBold,
        fontSize = 18.sp,
        lineHeight = 24.sp
    ),
    titleMedium = TextStyle(
        fontFamily = FontFamily.Default,
        fontWeight = FontWeight.Medium,
        fontSize = 16.sp,
        lineHeight = 22.sp
    ),
    bodyLarge = TextStyle(
        fontFamily = FontFamily.Default,
        fontWeight = FontWeight.Normal,
        fontSize = 15.sp,
        lineHeight = 22.sp
    ),
    bodyMedium = TextStyle(
        fontFamily = FontFamily.Default,
        fontWeight = FontWeight.Normal,
        fontSize = 14.sp,
        lineHeight = 20.sp
    ),
    labelSmall = TextStyle(
        fontFamily = FontFamily.Monospace,
        fontWeight = FontWeight.Medium,
        fontSize = 11.sp,
        lineHeight = 16.sp
    )
)
