# pip install Pillow

from PIL import Image

def add_border_to_mac_app_icon(input_file, output_file="app-icon-with-border.png"):
    # Open the existing icon
    original_icon = Image.open(input_file)
    
    # Ensure the input image is 1024x1024
    if original_icon.size != (1024, 1024):
        raise ValueError("Input image must be 1024x1024 pixels")
    
    # Create a new 1024x1024 transparent image
    new_icon = Image.new("RGBA", (1024, 1024), (0, 0, 0, 0))
    
    # Calculate dimensions
    border = int(1024 * 0.05)  # 5% border on each side
    icon_size = 1024 - (2 * border)  # 90% of total size
    
    # Resize the original icon to 90% of its size
    resized_icon = original_icon.resize((icon_size, icon_size), Image.Resampling.LANCZOS)
    
    # Paste the resized icon onto the new image, centered
    new_icon.paste(resized_icon, (border, border), resized_icon)
    
    # Save the new icon
    new_icon.save(output_file, "PNG")
    print(f"Icon with border saved as {output_file}")

# Usage
input_icon_path = "app-icon-rounded.png"
add_border_to_mac_app_icon(input_icon_path)