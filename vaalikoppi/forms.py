from django import forms

from vaalikoppi.models import NormalVoting, RankedChoiceVoting

fields = [
    "voting_name",
    "voting_description",
    "max_votes",
    "is_password_protected",
    "voting_password",
    "hide_vote_counts",
]
labels = {
    "voting_name": "Äänestyksen nimi",
    "voting_description": "Äänestyksen kuvaus",
    "max_votes": "Ääniä käytössä",
    "is_password_protected": "Äänestyskohtainen salasana vaaditaan",
    "voting_password": "Äänestyskohtainen salasana",
    "hide_vote_counts": "Älä näytä äänten määrää",
}


class VotingForm(forms.ModelForm):
    class Meta:
        model = NormalVoting
        fields = fields
        labels = labels


class RankedChoiceVotingForm(forms.ModelForm):
    class Meta:
        model = RankedChoiceVoting
        fields = fields
        labels = labels
