abstract class Breakpoints {
  static const double compact = 600.0;
  static const double medium = 1024.0;
}

enum ScreenSize {
  compact,
  medium,
  expanded,
}

ScreenSize getScreenSize(double width) {
  if (width < Breakpoints.compact) {
    return ScreenSize.compact;
  } else if (width <= Breakpoints.medium) {
    return ScreenSize.medium;
  } else {
    return ScreenSize.expanded;
  }
}
