import 'package:flutter/material.dart';
import 'breakpoints.dart';

class ResponsiveLayoutBuilder extends StatelessWidget {
  final Widget Function(BuildContext context) compact;
  final Widget Function(BuildContext context)? medium;
  final Widget Function(BuildContext context)? expanded;

  const ResponsiveLayoutBuilder({
    super.key,
    required this.compact,
    this.medium,
    this.expanded,
  });

  @override
  Widget build(BuildContext context) {
    final width = MediaQuery.sizeOf(context).width;
    final size = getScreenSize(width);

    switch (size) {
      case ScreenSize.compact:
        return compact(context);
      case ScreenSize.medium:
        return (medium ?? expanded ?? compact)(context);
      case ScreenSize.expanded:
        return (expanded ?? medium ?? compact)(context);
    }
  }
}
