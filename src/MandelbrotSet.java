import java.awt.image.BufferedImage;
import java.io.File;
import java.io.IOException;

import javax.imageio.ImageIO;

public class MandelbrotSet {

	public static void main(String[] args) {

		double progress = 0;
		double amount;

		// I think there might be some stuff mixed up in the code but it works
		// not sure though

		double[][] interestingPoints = { { -0.75, 0 }, { -0.77568377, 0.13646737 } };

		// important settings-------------------------------------------------------------
		int pointNumber = 1;
		int quality = 2; // 0 = very low, 1 = low, 2 = medium, 3 = HIGH, 4 = ultra, any number bigger than 5 = custom
		double zoomSpeed = 1.1; // >1
		int frames = 100;
		int width = 1920;
		//................................................................................

		// less important settings
		boolean advancedIndicators = true;
		double forceCenterX = interestingPoints[pointNumber][0];
		double forceCenterY = interestingPoints[pointNumber][1];
		boolean imageMode = true;
		int height = 1080;
		boolean locked = true;
		float ratio = 2 / 3f;  // .2 for editor 2/3 for image
		double zoom = 1;
		int iterations;
		double threshold = 100;
		char low = ' ';
		char HIGH = '#';

		if (quality == 0) {
			iterations = 20;
		} else if (quality == 1) {
			iterations = 50;
		} else if (quality == 2) {
			iterations = 100;
		} else if (quality == 3) {
			iterations = 500;
		} else if (quality == 4) {
			iterations = 1000;
		} else {
			iterations = quality;
		}

		double offsetX;
		double offsetY;
		if (locked) {
			height = (int) ((float) width * ratio);
		}

		// TIME
		long startTime = System.currentTimeMillis();

		for (int frameCounter = 0; frameCounter < frames; frameCounter++) {
			// progress = 0;
			BufferedImage image = new BufferedImage(width, height, BufferedImage.TYPE_INT_RGB);
			File f = null;

			// the printed image is 3 * 2 (from x = -3 to x = 1 and
			double stepSizeX = (3 / (float) width) / zoom;
			double stepSizeY = (2 / (float) height) / zoom;

			offsetX = forceCenterX - width / 2 * stepSizeX;
			offsetY = -(forceCenterY - height / 2 * stepSizeY);

			// calculate coords using stepSize and hardcoded corner coords:
			// create an array of complex numbers, where the position of each sample will be stored
			CNumber[][] coords = new CNumber[width][height];


			for (int i = 0; i < width; i++) {
				for (int j = 0; j < height; j++) {
					coords[i][j] = new CNumber(); // fill the array
					coords[i][j].real = offsetX + stepSizeX * i; // calculate the position on the real numberline
					coords[i][j].imag = offsetY - stepSizeY * j; // calculate the position on the imaginary numberline
				}
			}

			// calculate values
			double[][] values = new double[width][height]; // new array of booleans for the drawing
			for (int i = 0; i < width; i++) {
				for (int j = 0; j < height; j++) {
					values[i][j] = checkMandelbrot(coords[i][j], iterations, threshold); // check if the number is inside of th set
				}

			}

			if (imageMode) {

				createImage(image, frameCounter, f, width, height, values);

			} else {
				// draw
				draw(low, HIGH, width, height, values);
			}

			System.out.println("------------------------Frame " + frameCounter + " finished------------------------");

			zoom *= zoomSpeed;
		}

		// TIME
		long endtime = System.currentTimeMillis();
		long completionTimeLong = endtime - startTime;
		double completionTimeSec = (double) completionTimeLong / 1000.0;
		System.out.println("Calculated " + frames + " frame/s. Process took " + completionTimeSec + "s");

	}

	static void draw(char low, char HIGH, int width, int height, double[][] values) {
		// a method to draw a filled rectangle of size width * height
		// each cell can be low or HIGH, and it will show the corresponding char in each cell
		String line;

		for (int i = 0; i < height; i++) {
			line = "";
			// for every line:
			for (int j = 0; j < width; j++) {
				// for every char:
				double value = values[j][i];
				if (value >= 1) {
					line += HIGH;
				} else {
					line += low;
				}
			}

			System.out.println(line);
		}
	}

	static double checkMandelbrot(CNumber number, int iterations, double threshold) {

		// start
		CNumber n = new CNumber();
		CNumber c = number;

		// first
		n = CNumber.add(n, c);

		for (int i = 0; i < iterations; i++) {
			n = CNumber.add(CNumber.multiply(n, n), c);		// CNumber.multiply(n, n)
		}

		// System.out.println(n.real + " " + n.imag);

		if (n.real < threshold && n.imag < threshold) {
			return 1;
		} else {
			return 0;
		}


	}

	static void createImage(BufferedImage image, int counter, File f, int width, int height, double[][] values) {


		System.out.println("Frame: " + counter + " | Started creating image...");



		int p0 = getColorAsInt(0, 0, 0, 0);
		int p1 = getColorAsInt(0, 50, 50, 50);
		int p2 = getColorAsInt(0, 100, 100, 100);
		int p3 = getColorAsInt(0, 150, 150, 150);
		int p4 = getColorAsInt(0, 200, 200, 200);
		int pMax = getColorAsInt(0, 255, 255, 255);

		int threshold1 = 10;
		int threshold2 = 20;

		for (int i = 0; i < width; i++) {
			for (int j = 0; j < height; j++) {
				if (values[i][j] >= 1) {
					image.setRGB(i, j, p0);
				} else {
					image.setRGB(i, j, pMax);
				}
			}
		}

		try {
			f = new File("sequence\\Sequence" + counter + ".png");
			ImageIO.write(image, "png", f);
			System.out.println(f.getAbsolutePath());
		} catch (IOException e) {
			System.out.println(e);
		}

		System.out.println("Frame: " + counter + " | Finished creating image.");
	}

	public static int getColorAsInt(int a, int r, int g, int b) {

		int p1 = (a << 24) | (r << 16) | (g << 8) | b;

		return p1;
	}

}

// ^ originaler italienischer spaghetti code ^