#import "/src/boxes.typ": *
#import "/src/responsive-image.typ": *
#import "/src/code.typ": *


#note[This section assumes, that you have a working .NET 8 setup. If that is not the case, please follow #link("https://learn.microsoft.com/en-us/dotnet/core/install/")[the official Microsoft guide] to install .NET on your machine.]

\
Let's start with a fresh ASP.NET project by executing `dotnet new webapi` in a new directory. The starter project has a single endpoint defined in `Program.cs`. We can try that endpoint out in the swagger UI. Start up the service (`dotnet run`) and follow the link printed in the terminal. If the page is empty, navigate to `/swagger`. In the swagger UI, expand the `/weatherforecast` endpoint, press "Try it out", then "Execute". This will send an HTTP request to the running ASP.NET service and return made up weather data.

== New service endpoint

We will define a new endpoint to compile our Oicana template to a PDF and return the PDF file to the user.

\
1. Create a new directory in the .NET project called `templates` and copy `example-0.1.0.zip` into that directory.
2. Add the #link("https://www.nuget.org/packages/Oicana#readme-body-tab")[`Oicana` NuGet package] as a dependency with `dotnet add package Oicana --prerelease`.
3. Read the template file and prepare it for compilation at the beginning of `Program.cs`:

  \
  #code("Part of Program.cs", "csharp-integration-program-load-file")[
    ```cs
    using Oicana.Config;
    using Oicana.Template;

    var templateFile =
        await File.ReadAllBytesAsync("templates/example-0.1.0.zip");
    var template = new Template(templateFile);
    ```]

  \
4. Replace the generated `/weatherforecast` endpoint with the following:

  #code(
    "Part of Program.cs",
    "csharp-integration-program-compile-endpoint",
  )[
    ```cs
    app.MapPost("compile", () =>
    {
        var stream = template.Compile([], [], CompilationOptions.Pdf());
        var now = DateTimeOffset.Now;
        return Results.File(
            fileStream: stream,
            contentType: "application/pdf",
            fileDownloadName: $"example_{now:yyyy_MM_dd_HH_mm_ss_ffff}.pdf"
        );
    })
    .WithOpenApi();
    ```]

  This code defines a new POST endpoint at `/compile`. For every request, it compiles the template to PDF with two empty input lists and returns the file.

After restarting the service and refreshing the swagger UI, you should see the new endpoint. Open up the endpoint description and click "Try it out" and "Execute" to send a request to the server. You should see a successful response with a download button for the PDF file.

\
#responsive-image(
  "/assets/swagger_response.png",
  "The 200 response from calling an endpoint in Swagger UI. The response includes a link called \"Download file\".",
)

== About performance

The PDF generation should not take longer than a couple of milliseconds. You can look at the request duration in the network tab of your browser's debugging tools for an estimation. The first request to an APS.NET service can be significantly slower than later ones, because ASP.NET does some preparation during the first request.

\
For a better measurement of the compilation speed on your machine, you can use a #link("https://learn.microsoft.com/en-us/dotnet/api/system.diagnostics.stopwatch")[`Stopwatch`] in the endpoint code.


\
Next up: add dynamic inputs to the Oicana template.
