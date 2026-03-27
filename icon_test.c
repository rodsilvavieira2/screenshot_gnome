#include <gtk/gtk.h>
#include <stdio.h>

int main(int argc, char **argv) {
    gtk_init();
    
    GResource *res = g_resource_load("src/resources.gresource", NULL);
    if (!res) {
        printf("Failed to load resource\n");
        return 1;
    }
    g_resources_register(res);
    
    GdkDisplay *display = gdk_display_get_default();
    GtkIconTheme *theme = gtk_icon_theme_get_for_display(display);
    
    gtk_icon_theme_add_resource_path(theme, "/org/example/ScreenshotGnome/icons");
    
    gboolean has_icon = gtk_icon_theme_has_icon(theme, "app-object-select-symbolic");
    printf("Has icon 'app-object-select-symbolic': %s\n", has_icon ? "yes" : "no");
    
    gboolean has_icon_cross = gtk_icon_theme_has_icon(theme, "app-process-stop-symbolic");
    printf("Has icon 'app-process-stop-symbolic': %s\n", has_icon_cross ? "yes" : "no");

    return 0;
}
