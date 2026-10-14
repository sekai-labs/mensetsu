import 'package:flutter/material.dart';
import 'app_colors.dart';

abstract class AppTypography {
  static const String sansFont = 'Inter';
  static const String monoFont = 'JetBrains Mono';

  static TextStyle h1(BuildContext context, {Color? color}) => TextStyle(
        fontFamily: sansFont,
        fontSize: 22,
        fontWeight: FontWeight.w700,
        letterSpacing: -0.4,
        color: color ?? Theme.of(context).colorScheme.onSurface,
        height: 1.25,
      );

  static TextStyle h2(BuildContext context, {Color? color}) => TextStyle(
        fontFamily: sansFont,
        fontSize: 17,
        fontWeight: FontWeight.w600,
        letterSpacing: -0.2,
        color: color ?? Theme.of(context).colorScheme.onSurface,
        height: 1.3,
      );

  static TextStyle h3(BuildContext context, {Color? color}) => TextStyle(
        fontFamily: sansFont,
        fontSize: 14,
        fontWeight: FontWeight.w600,
        color: color ?? Theme.of(context).colorScheme.onSurface,
        height: 1.35,
      );

  static TextStyle body(BuildContext context, {Color? color}) => TextStyle(
        fontFamily: sansFont,
        fontSize: 13,
        fontWeight: FontWeight.w400,
        color: color ?? Theme.of(context).colorScheme.onSurface,
        height: 1.45,
      );

  static TextStyle bodyMedium(BuildContext context, {Color? color}) => TextStyle(
        fontFamily: sansFont,
        fontSize: 13,
        fontWeight: FontWeight.w500,
        color: color ?? Theme.of(context).colorScheme.onSurface,
        height: 1.4,
      );

  static TextStyle caption(BuildContext context, {Color? color}) => TextStyle(
        fontFamily: sansFont,
        fontSize: 11,
        fontWeight: FontWeight.w400,
        color: color ?? AppColors.slate500,
        height: 1.35,
      );

  static TextStyle mono(BuildContext context, {double fontSize = 12, Color? color, FontWeight? fontWeight}) => TextStyle(
        fontFamily: monoFont,
        fontSize: fontSize,
        fontWeight: fontWeight ?? FontWeight.w500,
        color: color ?? Theme.of(context).colorScheme.onSurface,
        height: 1.35,
      );

  static TextStyle badge({Color? color}) => TextStyle(
        fontFamily: monoFont,
        fontSize: 10,
        fontWeight: FontWeight.w700,
        letterSpacing: 0.5,
        color: color ?? AppColors.primary,
      );
}
