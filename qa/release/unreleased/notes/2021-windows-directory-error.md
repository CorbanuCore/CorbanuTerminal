## Windows sandbox

Windows elevated-sandbox launch failures with error 267 now explain that the working directory is invalid or inaccessible to the sandbox account. The error remains a launch failure, with no credential refresh or retry outside the sandbox. Documentation covers the full-access limitation at Moderate/Aggressive; this change does not fix that limitation (#2021).
