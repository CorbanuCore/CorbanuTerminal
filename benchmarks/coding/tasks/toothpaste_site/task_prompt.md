You are building a static ecommerce landing page and mock checkout.

Product: LumaDent Night Mint Whitening Gel, a fictional premium toothpaste.

Goal:
Create a good-looking, effective sale website that persuades a shopper to buy
one tube or a three-pack. Include a working mock checkout flow.

Image requirement:
Use the OpenAI Image API with model gpt-image-2 to generate exactly three final
site images. Save them locally under assets/generated or public/assets/generated.
Create image_manifest.json with filename, model, prompt, size, quality, and
created_at for each generated image. Never print or save API keys.

Website requirements:
- Static HTML/CSS/JS is preferred; no backend is required.
- No external image URLs or remote runtime assets.
- First viewport must show the product, price, CTA, and a generated visual.
- Include benefits, product details, trust/review content, and clear purchase options.
- Mock checkout must support add-to-cart, quantity changes, basic field
  validation, order summary, and success confirmation.
- Use restrained dental/beauty claims. Do not imply clinical proof, cure, or
  dentist endorsement unless it is explicitly fictional and non-medical.
- The site must work at desktop 1440px and mobile 390px widths.

OpenAI image generation:
- The environment provides OPENAI_API_KEY.
- Use model gpt-image-2 directly through the Image API generations endpoint or SDK equivalent.
- Do not echo the key, write it into files, or include it in image_manifest.json.
- Keep exactly three final generated images in the manifest. Do not include throwaway drafts in the manifest.

Definition of done:
- Open the site locally without a build service if possible.
- If you use a build tool, provide a script that starts the site.
- Run the visible checks available in the candidate.
- Fix failures until the benchmark harness's external verifier passes.
- Finish by saying BENCH_DONE.
