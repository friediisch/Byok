from PIL import Image, ImageDraw

def add_rounded_corners(input_file, output_file="rounded_corner_icon.png", corner_radius=100):
    # Open the image
    img = Image.open(input_file)

    # Ensure the input image is 1024x1024
    if img.size != (1024, 1024):
        raise ValueError("Input image must be 1024x1024 pixels")

    # Create a new image with transparent background
    rounded_img = Image.new("RGBA", (1024, 1024), (0, 0, 0, 0))

    # Create a mask for rounded corners
    mask = Image.new("L", (1024, 1024), 0)
    draw = ImageDraw.Draw(mask)

    # Draw a rounded rectangle on the mask
    draw.rounded_rectangle([(0, 0), (1024, 1024)], corner_radius, fill=255)

    # Apply the mask to the original image
    rounded_img.paste(img, (0, 0), mask)

    # Save the result
    rounded_img.save(output_file, "PNG")
    print(f"Rounded corner icon saved as {output_file}")

# Usage
input_icon_path = "path/to/your/original_icon.png"
add_rounded_corners(input_icon_path)

# Optionally, you can specify a different corner radius:
# add_rounded_corners(input_icon_path, corner_radius=150)